use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Seek, SeekFrom, Write},
    ops::{Bound, RangeBounds},
    path::{Path, PathBuf},
    sync::mpsc,
};

use bitvec::vec::BitVec;
use flate2::{Compression, bufread::MultiGzDecoder, write::GzEncoder};
use rand::{
    SeedableRng,
    distr::{Distribution, Uniform},
    rngs::SmallRng,
};
use tempfile::TempDir;
use thiserror::Error;
use tracing::{info, info_span, warn};

use crate::{
    cli::SubsampleArgs,
    fastq::{self, Record},
};

const VALID_PROBABILITY_RANGE: (Bound<f64>, Bound<f64>) =
    (Bound::Excluded(0.0), Bound::Excluded(1.0));

pub fn subsample(args: SubsampleArgs) -> Result<(), SubsampleError> {
    let r1_src = &args.r1_src;
    let r1_dst = &args.r1_dst;

    let r2_src = args.r2_src.as_ref();
    let r2_dst = args.r2_dst.as_ref();

    info!(command = "subsample", "fq");

    let rng = if let Some(seed) = args.seed {
        info!(seed = seed, "initializing rng from seed");
        SmallRng::seed_from_u64(seed)
    } else {
        info!("initializing rng from entropy");
        SmallRng::from_os_rng()
    };

    let r2 = (r2_src.map(|p| &**p), r2_dst.map(|p| &**p));

    if args.bin_by_tile || args.record_count_per_tile.is_some() {
        let tile_count_mode = if let Some(record_count_per_tile) = args.record_count_per_tile {
            TileCountMode::Explicit(record_count_per_tile)
        } else if let Some(probability) = args.probability {
            TileCountMode::FromProbability(probability)
        } else if let Some(record_count) = args.record_count {
            TileCountMode::FromRecordCount(record_count)
        } else {
            unreachable!();
        };
        subsample_by_tile(
            (r1_src, r1_dst),
            r2,
            rng,
            tile_count_mode,
            args.fast,
            args.sampling_threads,
            args.compression_threads,
            args.in_memory,
            args.temp_dir.as_deref(),
        )?;
    } else if let Some(probability) = args.probability {
        subsample_approximate((r1_src, r1_dst), r2, rng, probability)?;
    } else if let Some(record_count) = args.record_count {
        // Skip-ahead only works for single-end uncompressed input
        if args.fast && !is_gzipped(r1_src) && r2_src.is_none() {
            subsample_skip_ahead((r1_src, r1_dst), rng, record_count)?;
        } else {
            subsample_exact((r1_src, r1_dst), r2, rng, record_count)?;
        }
    } else {
        unreachable!();
    }

    info!("done");

    Ok(())
}

fn is_gzipped(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("gz")
}

fn subsample_approximate<Rng>(
    (r1_src, r1_dst): (&Path, &Path),
    (r2_src, r2_dst): (Option<&Path>, Option<&Path>),
    mut rng: Rng,
    probability: f64,
) -> Result<(), SubsampleError>
where
    Rng: rand::Rng,
{
    if !VALID_PROBABILITY_RANGE.contains(&probability) {
        return Err(SubsampleError::InvalidProbability(probability));
    }

    let mut r1 = fastq::fs::open(r1_src).map_err(|e| SubsampleError::OpenFile(e, r1_src.into()))?;
    let mut w1 =
        fastq::fs::create(r1_dst).map_err(|e| SubsampleError::CreateFile(e, r1_dst.into()))?;

    let span = info_span!("subsample_approximate", probability = probability);
    let _span_ctx = span.enter();

    let (n, total) = match (r2_src, r2_dst) {
        (Some(r2_src), Some(r2_dst)) => {
            info!("sampling paired end reads");

            let mut r2 =
                fastq::fs::open(r2_src).map_err(|e| SubsampleError::OpenFile(e, r2_src.into()))?;
            let mut w2 = fastq::fs::create(r2_dst)
                .map_err(|e| SubsampleError::CreateFile(e, r2_dst.into()))?;

            subsample_paired(
                (&mut r1, &mut w1),
                (&mut r2, &mut w2),
                &mut rng,
                probability,
            )?
        }
        (Some(_), None) => return Err(SubsampleError::MissingDestination("r2-dst")),
        (None, Some(_)) => return Err(SubsampleError::MissingSource("r2-src")),
        _ => {
            info!("sampling single end reads");
            subsample_single(&mut r1, &mut w1, &mut rng, probability)?
        }
    };

    let percentage = (n as f64) / (total as f64) * 100.0;
    info!("sampled {}/{} ({:.1}%) records", n, total, percentage);

    Ok(())
}

fn subsample_single<R, W, Rng>(
    reader: &mut fastq::io::Reader<R>,
    writer: &mut fastq::io::Writer<W>,
    rng: &mut Rng,
    p: f64,
) -> Result<(u64, u64), SubsampleError>
where
    R: BufRead,
    W: Write,
    Rng: rand::Rng,
{
    let mut record = Record::default();

    let mut n = 0;
    let mut total = 0;

    while reader.read_record(&mut record)? != 0 {
        let q: f64 = rng.random();

        if q <= p {
            writer.write_record(&record)?;
            n += 1;
        }

        total += 1;
    }

    Ok((n, total))
}

fn subsample_paired<R, S, W, X, Rng>(
    (r1, w1): (&mut fastq::io::Reader<R>, &mut fastq::io::Writer<W>),
    (r2, w2): (&mut fastq::io::Reader<S>, &mut fastq::io::Writer<X>),
    rng: &mut Rng,
    p: f64,
) -> Result<(u64, u64), SubsampleError>
where
    R: BufRead,
    S: BufRead,
    W: Write,
    X: Write,
    Rng: rand::Rng,
{
    let mut s1 = Record::default();
    let mut s2 = Record::default();

    let mut n = 0;
    let mut total = 0;

    loop {
        match (r1.read_record(&mut s1)?, r2.read_record(&mut s2)?) {
            (0, 0) => break,
            (0, len) if len > 0 => return Err(SubsampleError::UnexpectedEof("r1-src")),
            (len, 0) if len > 0 => return Err(SubsampleError::UnexpectedEof("r2-src")),
            (_, _) => {
                let q: f64 = rng.random();

                if q <= p {
                    w1.write_record(&s1)?;
                    w2.write_record(&s2)?;
                    n += 1;
                }

                total += 1;
            }
        }
    }

    Ok((n, total))
}

fn subsample_exact<Rng>(
    (r1_src, r1_dst): (&Path, &Path),
    (r2_src, r2_dst): (Option<&Path>, Option<&Path>),
    rng: Rng,
    mut record_count: u64,
) -> Result<(), SubsampleError>
where
    Rng: rand::Rng,
{
    let span = info_span!("subsample_exact", record_count = record_count);
    let _span_ctx = span.enter();

    info!("counting records");

    let actual_record_count = if let Some(index_count) = count_records_from_index(r1_src)? {
        if is_gzipped(r1_src) {
            // For gzipped input, trust the index (decompressing to cross-check
            // would negate the performance benefit of having an index).
            info!(actual_record_count = index_count, "counted records from index (gzipped; not cross-checked)");
            index_count
        } else {
            // For uncompressed input, cross-check against line count (cheap)
            let line_count = count_lines(r1_src)?;
            let file_count = line_count / 4;

            if index_count != file_count {
                warn!(
                    "index record count ({}) differs from file record count ({}); using file count (index may be stale)",
                    index_count, file_count
                );
                file_count
            } else {
                info!(actual_record_count = index_count, "counted records from index (verified)");
                index_count
            }
        }
    } else {
        let line_count = count_lines(r1_src)?;
        let count = line_count / 4;
        info!(actual_record_count = count, "counted records");
        count
    };

    if actual_record_count == 0 {
        info!("input is empty; producing empty output");
        fastq::fs::create(r1_dst).map_err(|e| SubsampleError::CreateFile(e, r1_dst.into()))?;
        if let Some(r2_dst) = r2_dst {
            fastq::fs::create(r2_dst).map_err(|e| SubsampleError::CreateFile(e, r2_dst.into()))?;
        }
        return Ok(());
    }

    let n = u64::try_from(actual_record_count)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if record_count > n {
        warn!(
            "record count ({}) > r1-src record count ({}). Using record-count = {} instead.",
            record_count, n, n
        );

        record_count = n;
    }

    info!("building filter");
    let bitmap = build_filter(rng, actual_record_count, record_count)?;
    info!("built filter");

    let mut r1 = fastq::fs::open(r1_src).map_err(|e| SubsampleError::OpenFile(e, r1_src.into()))?;
    let mut w1 =
        fastq::fs::create(r1_dst).map_err(|e| SubsampleError::CreateFile(e, r1_dst.into()))?;

    match (r2_src, r2_dst) {
        (Some(r2_src), Some(r2_dst)) => {
            info!("sampling paired end reads");

            let mut r2 =
                fastq::fs::open(r2_src).map_err(|e| SubsampleError::OpenFile(e, r2_src.into()))?;
            let mut w2 = fastq::fs::create(r2_dst)
                .map_err(|e| SubsampleError::CreateFile(e, r2_dst.into()))?;

            subsample_exact_paired((&mut r1, &mut w1), (&mut r2, &mut w2), &bitmap)?;
        }
        (Some(_), None) => return Err(SubsampleError::MissingDestination("r2-dst")),
        (None, Some(_)) => return Err(SubsampleError::MissingSource("r2-src")),
        (None, None) => {
            info!("sampling single end reads");
            subsample_exact_single(&mut r1, &mut w1, &bitmap)?;
        }
    }

    let percentage = (record_count as f64) / (actual_record_count as f64) * 100.0;
    info!(
        "sampled {}/{} ({:.1}%) records",
        record_count, actual_record_count, percentage
    );

    Ok(())
}

fn count_lines<P>(src: P) -> io::Result<usize>
where
    P: AsRef<Path>,
{
    const LINE_FEED: u8 = b'\n';

    let mut reader = open_maybe_gz(src)?;
    let mut n = 0;

    loop {
        let buf = reader.fill_buf()?;

        if buf.is_empty() {
            break;
        }

        n += bytecount::count(buf, LINE_FEED);

        let len = buf.len();
        reader.consume(len);
    }

    Ok(n)
}

/// Attempt to count records using a `.fai` index file.
///
/// Looks for `<src>.fai` (e.g., `reads.fq.fai` or `reads.fq.gz.fai`).
/// The samtools FASTQ index format has one line per record, so the number
/// of lines in the index equals the record count.
///
/// Returns `Ok(Some(count))` if an index was found and read successfully,
/// `Ok(None)` if no index exists, or an error if the index exists but
/// cannot be read.
fn count_records_from_index<P>(src: P) -> io::Result<Option<usize>>
where
    P: AsRef<Path>,
{
    let mut index_path = src.as_ref().as_os_str().to_owned();
    index_path.push(".fai");
    let index_path = PathBuf::from(index_path);

    if !index_path.exists() {
        return Ok(None);
    }

    info!(index = %index_path.display(), "found FASTQ index");

    const LINE_FEED: u8 = b'\n';

    let file = File::open(&index_path)?;
    let mut reader = BufReader::new(file);
    let mut n = 0;

    loop {
        let buf = reader.fill_buf()?;

        if buf.is_empty() {
            break;
        }

        n += bytecount::count(buf, LINE_FEED);

        let len = buf.len();
        reader.consume(len);
    }

    Ok(Some(n))
}

fn open_maybe_gz<P>(src: P) -> io::Result<Box<dyn BufRead>>
where
    P: AsRef<Path>,
{
    let path = src.as_ref();
    let extension = path.extension();
    let reader = File::open(path).map(BufReader::new)?;

    match extension.and_then(|ext| ext.to_str()) {
        Some("gz") => {
            let decoder = MultiGzDecoder::new(reader);
            Ok(Box::new(BufReader::new(decoder)))
        }
        _ => Ok(Box::new(reader)),
    }
}

fn build_filter<Rng>(
    mut rng: Rng,
    src_record_count: usize,
    dst_record_count: u64,
) -> Result<BitVec, SubsampleError>
where
    Rng: rand::Rng,
{
    let mut bitmap = BitVec::new();
    bitmap.resize(src_record_count, false);

    let distribution =
        Uniform::new(0, src_record_count).map_err(SubsampleError::InvalidUniformRange)?;

    let mut n = 0;

    while n < dst_record_count {
        let i = distribution.sample(&mut rng);

        if !bitmap[i] {
            bitmap.set(i, true);
            n += 1;
        }
    }

    Ok(bitmap)
}

fn subsample_exact_single<R, W>(
    reader: &mut fastq::io::Reader<R>,
    writer: &mut fastq::io::Writer<W>,
    bitmap: &BitVec,
) -> Result<(), SubsampleError>
where
    R: BufRead,
    W: Write,
{
    let mut record = Record::default();
    let mut i = 0;

    while reader.read_record(&mut record)? != 0 {
        if bitmap[i] {
            writer.write_record(&record)?;
        }

        i += 1;
    }

    Ok(())
}

fn subsample_exact_paired<R, S, W, X>(
    (r1, w1): (&mut fastq::io::Reader<R>, &mut fastq::io::Writer<W>),
    (r2, w2): (&mut fastq::io::Reader<S>, &mut fastq::io::Writer<X>),
    bitmap: &BitVec,
) -> Result<(), SubsampleError>
where
    R: BufRead,
    S: BufRead,
    W: Write,
    X: Write,
{
    let mut s1 = Record::default();
    let mut s2 = Record::default();

    let mut i = 0;

    loop {
        match (r1.read_record(&mut s1)?, r2.read_record(&mut s2)?) {
            (0, 0) => break,
            (0, len) if len > 0 => return Err(SubsampleError::UnexpectedEof("r1-src")),
            (len, 0) if len > 0 => return Err(SubsampleError::UnexpectedEof("r2-src")),
            (_, _) => {
                if bitmap[i] {
                    w1.write_record(&s1)?;
                    w2.write_record(&s2)?;
                }

                i += 1;
            }
        }
    }

    Ok(())
}

fn subsample_skip_ahead<Rng>(
    (r1_src, r1_dst): (&Path, &Path),
    mut rng: Rng,
    target_count: u64,
) -> Result<(), SubsampleError>
where
    Rng: rand::Rng,
{
    let span = info_span!("subsample_skip_ahead", target_count);
    let _span_ctx = span.enter();

    let r1_size = std::fs::metadata(r1_src)
        .map_err(|e| SubsampleError::OpenFile(e, r1_src.into()))?
        .len();

    // Estimate record count from first 100 records
    let avg_r1_size = estimate_avg_record_size(r1_src)?;
    let estimated_total = (r1_size as f64 / avg_r1_size) as u64;

    info!(
        r1_size,
        avg_r1_size,
        estimated_total,
        "estimated record count"
    );

    let target = target_count.min(estimated_total);
    let mut r1_buf = Vec::new();

    let r1_selected = skip_ahead_sample_file(r1_src, r1_size, avg_r1_size, target, &mut rng)?;

    for (_, data) in &r1_selected {
        r1_buf.extend_from_slice(data);
    }

    info!(selected = r1_selected.len(), "skip-ahead sampling complete");

    write_output_data(&r1_buf, r1_dst, 1)?;
    
    let percentage = if estimated_total > 0 {
        r1_selected.len() as f64 / estimated_total as f64 * 100.0
    } else {
        0.0
    };
    info!(
        "sampled ~{}/{} ({:.1}%) records",
        r1_selected.len(),
        estimated_total,
        percentage
    );

    Ok(())
}

enum TileCountMode {
    Explicit(u64),
    FromRecordCount(u64),
    FromProbability(f64),
}

/// Parses an Illumina read header to extract (lane, tile) as a packed u64 key.
///
/// Expected format: `@<instrument>:<run>:<flowcell>:<lane>:<tile>:<x>:<y>`
fn parse_tile_bin(name: &[u8]) -> Option<u64> {
    let name = if name.first() == Some(&b'@') {
        &name[1..]
    } else {
        name
    };

    // Strip description (everything after first space)
    let name = name.split(|&b| b == b' ').next()?;

    let mut parts = name.split(|&b| b == b':');

    // Skip instrument, run, flowcell (fields 0-2)
    parts.next()?;
    parts.next()?;
    parts.next()?;

    let lane_bytes = parts.next()?;
    let tile_bytes = parts.next()?;

    let lane: u32 = std::str::from_utf8(lane_bytes).ok()?.parse().ok()?;
    let tile: u32 = std::str::from_utf8(tile_bytes).ok()?.parse().ok()?;

    Some((lane as u64) << 32 | (tile as u64))
}

struct TileResult {
    r1_data: Vec<u8>,
    r2_data: Option<Vec<u8>>,
}

enum OutputWriter {
    Plain(BufWriter<File>),
    Gz(GzEncoder<BufWriter<File>>),
}

impl OutputWriter {
    fn create(dst: &Path) -> io::Result<Self> {
        let file = BufWriter::new(File::create(dst)?);
        if is_gzipped(dst) {
            Ok(OutputWriter::Gz(GzEncoder::new(file, Compression::default())))
        } else {
            Ok(OutputWriter::Plain(file))
        }
    }

    fn finish(self) -> io::Result<()> {
        match self {
            OutputWriter::Plain(mut w) => w.flush(),
            OutputWriter::Gz(w) => {
                w.finish()?;
                Ok(())
            }
        }
    }
}

impl Write for OutputWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            OutputWriter::Plain(w) => w.write(buf),
            OutputWriter::Gz(w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            OutputWriter::Plain(w) => w.flush(),
            OutputWriter::Gz(w) => w.flush(),
        }
    }
}

struct TileInfo {
    bin_key: u64,
    record_count: usize,
    r1_path: PathBuf,
    r2_path: Option<PathBuf>,
}

struct TileWriter {
    r1: fastq::io::Writer<BufWriter<File>>,
    r1_path: PathBuf,
    r2: Option<fastq::io::Writer<BufWriter<File>>>,
    r2_path: Option<PathBuf>,
    record_count: usize,
}

fn write_tile_temp_files(
    r1_src: &Path,
    r2_src: Option<&Path>,
    temp_dir: &Path,
) -> Result<(Vec<TileInfo>, u64), SubsampleError> {
    let mut r1 = fastq::fs::open(r1_src).map_err(|e| SubsampleError::OpenFile(e, r1_src.into()))?;
    let mut r1_rec = Record::default();

    let mut r2_reader = r2_src
        .map(|p| fastq::fs::open(p).map_err(|e| SubsampleError::OpenFile(e, p.into())))
        .transpose()?;
    let mut r2_rec = Record::default();

    let paired = r2_src.is_some();
    let mut tile_writers: HashMap<u64, TileWriter> = HashMap::new();
    let mut parse_failures: u64 = 0;

    loop {
        let r1_len = r1.read_record(&mut r1_rec)?;
        let r2_len = if let Some(r2) = r2_reader.as_mut() {
            r2.read_record(&mut r2_rec)?
        } else {
            0
        };

        match (r1_len, r2_len, paired) {
            (0, 0, _) | (0, _, false) => break,
            (0, _, true) => return Err(SubsampleError::UnexpectedEof("r1-src")),
            (_, 0, true) => return Err(SubsampleError::UnexpectedEof("r2-src")),
            _ => {}
        }

        if let Some(bin_key) = parse_tile_bin(r1_rec.name()) {
            if !tile_writers.contains_key(&bin_key) {
                let r1_path = temp_dir.join(format!("{bin_key}.r1.fq"));
                let r1_file = BufWriter::new(
                    File::create(&r1_path)
                        .map_err(|e| SubsampleError::CreateFile(e, r1_path.clone()))?,
                );
                let (r2_writer, r2_path) = if paired {
                    let p = temp_dir.join(format!("{bin_key}.r2.fq"));
                    let f = BufWriter::new(
                        File::create(&p)
                            .map_err(|e| SubsampleError::CreateFile(e, p.clone()))?,
                    );
                    (Some(fastq::io::Writer::new(f)), Some(p))
                } else {
                    (None, None)
                };
                tile_writers.insert(bin_key, TileWriter {
                    r1: fastq::io::Writer::new(r1_file),
                    r1_path,
                    r2: r2_writer,
                    r2_path,
                    record_count: 0,
                });
            }
            let tw = tile_writers.get_mut(&bin_key).unwrap();

            tw.r1.write_record(&r1_rec)?;
            if let Some(r2w) = tw.r2.as_mut() {
                r2w.write_record(&r2_rec)?;
            }
            tw.record_count += 1;
        } else {
            parse_failures += 1;
        }
    }

    // Collect TileInfo (writers are dropped here, flushing buffers)
    let mut tiles: Vec<TileInfo> = tile_writers
        .into_iter()
        .map(|(key, tw)| TileInfo {
            bin_key: key,
            record_count: tw.record_count,
            r1_path: tw.r1_path,
            r2_path: tw.r2_path,
        })
        .collect();
    // Sort by bin key for deterministic processing order
    tiles.sort_unstable_by_key(|t| t.bin_key);

    Ok((tiles, parse_failures))
}

fn subsample_by_tile<Rng>(
    (r1_src, r1_dst): (&Path, &Path),
    (r2_src, r2_dst): (Option<&Path>, Option<&Path>),
    mut rng: Rng,
    mode: TileCountMode,
    fast: bool,
    sampling_threads: usize,
    compression_threads: usize,
    in_memory: bool,
    temp_dir_path: Option<&Path>,
) -> Result<(), SubsampleError>
where
    Rng: rand::Rng + Send,
{
    let span = info_span!("subsample_by_tile");
    let _span_ctx = span.enter();

    // Validate paired args
    match (r2_src, r2_dst) {
        (Some(_), None) => return Err(SubsampleError::MissingDestination("r2-dst")),
        (None, Some(_)) => return Err(SubsampleError::MissingSource("r2-src")),
        _ => {}
    }

    if in_memory {
        return subsample_by_tile_in_memory(
            (r1_src, r1_dst),
            (r2_src, r2_dst),
            rng,
            mode,
            compression_threads,
        );
    }

    // Create temp directory
    let temp_dir = if let Some(parent) = temp_dir_path {
        TempDir::new_in(parent).map_err(SubsampleError::TempDir)?
    } else {
        TempDir::new().map_err(SubsampleError::TempDir)?
    };
    info!(temp_dir = %temp_dir.path().display(), "created temp directory");

    // First pass: write per-tile temp files
    info!("first pass: writing per-tile temp files");
    let (tiles, parse_failures) = write_tile_temp_files(r1_src, r2_src, temp_dir.path())?;

    let total_records: usize = tiles.iter().map(|t| t.record_count).sum();
    let num_bins = tiles.len();

    info!(
        total_records,
        parse_failures,
        bins = num_bins,
        "binning complete"
    );

    if parse_failures > 0 {
        warn!(
            "{} records had headers that could not be parsed as Illumina format",
            parse_failures
        );
    }

    // Compute per-tile target
    let record_count_per_tile = match mode {
        TileCountMode::Explicit(n) => n as usize,
        TileCountMode::FromRecordCount(target) => {
            if num_bins == 0 { 0 } else { target as usize / num_bins }
        }
        TileCountMode::FromProbability(p) => {
            if num_bins == 0 {
                0
            } else {
                (p * total_records as f64 / num_bins as f64).floor() as usize
            }
        }
    };

    info!(record_count_per_tile, "computed per-tile target");

    if record_count_per_tile == 0 {
        warn!("per-tile record count is 0; output will be empty");
        write_output_data(&[], r1_dst, compression_threads)?;
        if let Some(r2_dst) = r2_dst {
            write_output_data(&[], r2_dst, compression_threads)?;
        }
        return Ok(());
    }

    // Filter tiles
    let retained_tiles: Vec<&TileInfo> = tiles
        .iter()
        .filter(|t| t.record_count >= record_count_per_tile)
        .collect();

    let retained_count = retained_tiles.len();
    let discarded_count = num_bins - retained_count;
    info!(
        retained_bins = retained_count,
        discarded_bins = discarded_count,
        "filtered bins"
    );

    if retained_count == 0 {
        warn!("no bins have enough records; output will be empty");
        write_output_data(&[], r1_dst, compression_threads)?;
        if let Some(r2_dst) = r2_dst {
            write_output_data(&[], r2_dst, compression_threads)?;
        }
        return Ok(());
    }

    // Sample tiles and stream results to a writer thread via channel.
    // This avoids holding all sampled data in memory at once.
    let paired = r2_src.is_some();
    let threads = sampling_threads.min(retained_count).max(1);

    info!(
        fast,
        sampling_threads = threads,
        "sampling {} tiles",
        retained_count
    );

    let base_seed: u64 = rng.random();
    let (tx, rx) = mpsc::sync_channel::<TileResult>(threads * 2);

    let selected_records = std::thread::scope(|s| -> Result<usize, SubsampleError> {
        // Writer thread: receives tile results and writes to output files
        let writer_handle = s.spawn(|| -> Result<usize, SubsampleError> {
            let mut r1_writer = OutputWriter::create(r1_dst)?;
            let mut r2_writer = if paired {
                Some(OutputWriter::create(r2_dst.unwrap())?)
            } else {
                None
            };
            let mut selected = 0usize;

            for result in rx {
                selected += result.r1_data.iter().filter(|&&b| b == b'\n').count() / 4;
                r1_writer.write_all(&result.r1_data)?;
                if let (Some(w), Some(data)) = (&mut r2_writer, result.r2_data) {
                    w.write_all(&data)?;
                }
            }

            r1_writer.finish()?;
            if let Some(w) = r2_writer {
                w.finish()?;
            }

            Ok(selected)
        });

        // Sampling threads: process tiles and send results through channel
        let chunk_size = (retained_count + threads - 1) / threads;
        let chunks: Vec<&[&TileInfo]> = retained_tiles.chunks(chunk_size).collect();

        let handles: Vec<_> = chunks
            .into_iter()
            .map(|chunk| {
                let tx = tx.clone();

                s.spawn(move || -> Result<(), SubsampleError> {
                    for tile in chunk {
                        let mut tile_rng =
                            SmallRng::seed_from_u64(base_seed.wrapping_add(tile.bin_key));

                        // Skip-ahead only works for single-end; paired-end
                        // always uses exact indexed sampling.
                        let (r1_data, r2_data) = if fast && !paired {
                            sample_tile_skip_ahead(tile, record_count_per_tile, &mut tile_rng)?
                        } else {
                            sample_tile_exact(tile, record_count_per_tile, &mut tile_rng)?
                        };

                        tx.send(TileResult { r1_data, r2_data }).unwrap();
                    }
                    Ok(())
                })
            })
            .collect();

        // Drop the original sender so the writer sees EOF when all clones are dropped
        drop(tx);

        for handle in handles {
            handle.join().unwrap()?;
        }

        writer_handle.join().unwrap()
    })?;

    let percentage = if total_records > 0 {
        selected_records as f64 / total_records as f64 * 100.0
    } else {
        0.0
    };
    info!(
        "sampled {}/{} ({:.1}%) records across {} tiles",
        selected_records, total_records, percentage, retained_count
    );

    Ok(())
}

struct InMemoryTile {
    r1_records: Vec<Vec<u8>>,
    r2_records: Vec<Vec<u8>>,
}

fn subsample_by_tile_in_memory<Rng>(
    (r1_src, r1_dst): (&Path, &Path),
    (r2_src, r2_dst): (Option<&Path>, Option<&Path>),
    mut rng: Rng,
    mode: TileCountMode,
    compression_threads: usize,
) -> Result<(), SubsampleError>
where
    Rng: rand::Rng,
{
    info!("using in-memory tile binning");

    let mut r1 = fastq::fs::open(r1_src).map_err(|e| SubsampleError::OpenFile(e, r1_src.into()))?;
    let mut r1_rec = Record::default();

    let mut r2_reader = r2_src
        .map(|p| fastq::fs::open(p).map_err(|e| SubsampleError::OpenFile(e, p.into())))
        .transpose()?;
    let mut r2_rec = Record::default();

    let paired = r2_src.is_some();
    let mut tiles: HashMap<u64, InMemoryTile> = HashMap::new();
    let mut parse_failures: u64 = 0;

    // First pass: read all records into memory, binned by tile
    loop {
        let r1_len = r1.read_record(&mut r1_rec)?;
        let r2_len = if let Some(r2) = r2_reader.as_mut() {
            r2.read_record(&mut r2_rec)?
        } else {
            0
        };

        match (r1_len, r2_len, paired) {
            (0, 0, _) | (0, _, false) => break,
            (0, _, true) => return Err(SubsampleError::UnexpectedEof("r1-src")),
            (_, 0, true) => return Err(SubsampleError::UnexpectedEof("r2-src")),
            _ => {}
        }

        if let Some(bin_key) = parse_tile_bin(r1_rec.name()) {
            let tile = tiles.entry(bin_key).or_insert_with(|| InMemoryTile {
                r1_records: Vec::new(),
                r2_records: Vec::new(),
            });
            tile.r1_records.push(r1_rec.as_ref().to_vec());
            if paired {
                tile.r2_records.push(r2_rec.as_ref().to_vec());
            }
        } else {
            parse_failures += 1;
        }
    }

    let total_records: usize = tiles.values().map(|t| t.r1_records.len()).sum();
    let num_bins = tiles.len();

    info!(
        total_records,
        parse_failures,
        bins = num_bins,
        "in-memory binning complete"
    );

    if parse_failures > 0 {
        warn!(
            "{} records had headers that could not be parsed as Illumina format",
            parse_failures
        );
    }

    // Compute per-tile target
    let record_count_per_tile = match mode {
        TileCountMode::Explicit(n) => n as usize,
        TileCountMode::FromRecordCount(target) => {
            if num_bins == 0 { 0 } else { target as usize / num_bins }
        }
        TileCountMode::FromProbability(p) => {
            if num_bins == 0 {
                0
            } else {
                (p * total_records as f64 / num_bins as f64).floor() as usize
            }
        }
    };

    info!(record_count_per_tile, "computed per-tile target");

    if record_count_per_tile == 0 {
        warn!("per-tile record count is 0; output will be empty");
        write_output_data(&[], r1_dst, compression_threads)?;
        if let Some(r2_dst) = r2_dst {
            write_output_data(&[], r2_dst, compression_threads)?;
        }
        return Ok(());
    }

    // Sample from retained tiles
    let mut r1_buf = Vec::new();
    let mut r2_buf = Vec::new();
    let mut retained_count = 0usize;

    for tile in tiles.values() {
        let count = tile.r1_records.len();
        if count < record_count_per_tile {
            continue;
        }

        retained_count += 1;

        // Select random indices
        let distribution = Uniform::new(0, count).map_err(SubsampleError::InvalidUniformRange)?;
        let mut selected = vec![false; count];
        let mut n = 0;
        while n < record_count_per_tile {
            let i = distribution.sample(&mut rng);
            if !selected[i] {
                selected[i] = true;
                n += 1;
            }
        }

        for (i, &sel) in selected.iter().enumerate() {
            if sel {
                r1_buf.extend_from_slice(&tile.r1_records[i]);
                if paired {
                    r2_buf.extend_from_slice(&tile.r2_records[i]);
                }
            }
        }
    }

    let discarded_count = num_bins - retained_count;
    info!(
        retained_bins = retained_count,
        discarded_bins = discarded_count,
        "filtered bins"
    );

    let selected_records = r1_buf.iter().filter(|&&b| b == b'\n').count() / 4;

    write_output_data(&r1_buf, r1_dst, compression_threads)?;
    if paired {
        write_output_data(&r2_buf, r2_dst.unwrap(), compression_threads)?;
    }

    let percentage = if total_records > 0 {
        selected_records as f64 / total_records as f64 * 100.0
    } else {
        0.0
    };
    info!(
        "sampled {}/{} ({:.1}%) records across {} tiles",
        selected_records, total_records, percentage, retained_count
    );

    Ok(())
}

/// Sample from an uncompressed FASTQ file using geometric jumps.
/// Returns Vec of (byte_position, record_bytes) for selected records.
fn skip_ahead_sample_file<Rng>(
    path: &Path,
    file_size: u64,
    avg_record_size: f64,
    target: u64,
    rng: &mut Rng,
) -> Result<Vec<(u64, Vec<u8>)>, SubsampleError>
where
    Rng: rand::Rng,
{
    if file_size == 0 || target == 0 {
        return Ok(Vec::new());
    }

    let mean_jump_bytes = file_size as f64 / target as f64;
    let mut file = BufReader::new(
        File::open(path).map_err(|e| SubsampleError::OpenFile(e, path.into()))?,
    );

    let mut results = Vec::with_capacity(target as usize);
    let mut current_pos: u64 = 0;
    let mut last_record_pos: Option<u64> = None;

    while (results.len() as u64) < target {
        // Generate exponential jump
        let jump = exponential_sample(rng, mean_jump_bytes) as u64;
        current_pos = current_pos.saturating_add(jump.max(avg_record_size as u64));

        if current_pos >= file_size {
            break;
        }

        match find_record_after(&mut file, current_pos)? {
            Some((record_pos, data)) => {
                // Skip if we landed on the same record as last time
                if last_record_pos == Some(record_pos) {
                    continue;
                }
                last_record_pos = Some(record_pos);
                current_pos = record_pos + data.len() as u64;
                results.push((record_pos, data));
            }
            None => break,
        }
    }

    Ok(results)
}

fn sample_tile_skip_ahead<Rng>(
    tile: &TileInfo,
    target: usize,
    rng: &mut Rng,
) -> Result<(Vec<u8>, Option<Vec<u8>>), SubsampleError>
where
    Rng: rand::Rng,
{
    let r1_size = std::fs::metadata(&tile.r1_path)
        .map_err(|e| SubsampleError::OpenFile(e, tile.r1_path.clone()))?
        .len();

    if r1_size == 0 {
        return Ok((Vec::new(), tile.r2_path.as_ref().map(|_| Vec::new())));
    }

    let avg_r1_size = r1_size as f64 / tile.record_count as f64;

    let r1_selected =
        skip_ahead_sample_file(&tile.r1_path, r1_size, avg_r1_size, target as u64, rng)?;

    let mut r1_buf = Vec::new();
    for (_, data) in &r1_selected {
        r1_buf.extend_from_slice(data);
    }

    let r2_buf = if let Some(r2_path) = &tile.r2_path {
        let r2_size = std::fs::metadata(r2_path)
            .map_err(|e| SubsampleError::OpenFile(e, r2_path.clone()))?
            .len();
        let avg_r2_size = r2_size as f64 / tile.record_count as f64;

        let mut buf = Vec::new();
        let mut r2_file = BufReader::new(
            File::open(r2_path).map_err(|e| SubsampleError::OpenFile(e, r2_path.clone()))?,
        );

        for &(r1_pos, _) in &r1_selected {
            // Compute proportional position in R2
            let r2_pos = (r1_pos as f64 / avg_r1_size * avg_r2_size) as u64;
            let r2_pos = r2_pos.min(r2_size.saturating_sub(1));

            if let Some((_, data)) = find_record_after(&mut r2_file, r2_pos)? {
                buf.extend_from_slice(&data);
            }
        }

        Some(buf)
    } else {
        None
    };

    Ok((r1_buf, r2_buf))
}

fn build_record_index(path: &Path) -> Result<Vec<u64>, SubsampleError> {
    let mut reader = BufReader::new(
        File::open(path).map_err(|e| SubsampleError::OpenFile(e, path.into()))?,
    );

    let mut offsets = Vec::new();
    let mut line_buf = Vec::new();
    let mut line_in_record = 0u8;

    loop {
        let pos = reader.stream_position()?;
        line_buf.clear();

        if reader.read_until(b'\n', &mut line_buf)? == 0 {
            break;
        }

        if line_in_record == 0 {
            offsets.push(pos);
        }

        line_in_record = (line_in_record + 1) % 4;
    }

    Ok(offsets)
}

fn read_record_at(reader: &mut BufReader<File>, offset: u64) -> io::Result<Vec<u8>> {
    reader.seek(SeekFrom::Start(offset))?;

    let mut data = Vec::new();
    let mut line_buf = Vec::new();

    for _ in 0..4 {
        line_buf.clear();
        if reader.read_until(b'\n', &mut line_buf)? == 0 {
            // If at EOF without newline, add one
            if !data.is_empty() && !data.ends_with(b"\n") {
                data.push(b'\n');
            }
            break;
        }
        data.extend_from_slice(&line_buf);
    }

    Ok(data)
}

fn sample_tile_exact<Rng>(
    tile: &TileInfo,
    target: usize,
    rng: &mut Rng,
) -> Result<(Vec<u8>, Option<Vec<u8>>), SubsampleError>
where
    Rng: rand::Rng,
{
    // Build index for R1
    let r1_index = build_record_index(&tile.r1_path)?;
    let actual_count = r1_index.len();
    let target = target.min(actual_count);

    // Select random indices
    let distribution = Uniform::new(0, actual_count).map_err(SubsampleError::InvalidUniformRange)?;
    let mut selected: Vec<bool> = vec![false; actual_count];
    let mut n = 0;
    while n < target {
        let i = distribution.sample(rng);
        if !selected[i] {
            selected[i] = true;
            n += 1;
        }
    }

    // Read selected R1 records
    let mut r1_file = BufReader::new(
        File::open(&tile.r1_path)
            .map_err(|e| SubsampleError::OpenFile(e, tile.r1_path.clone()))?,
    );
    let mut r1_buf = Vec::new();

    // Collect selected indices in sorted order for sequential reading
    let selected_indices: Vec<usize> = selected
        .iter()
        .enumerate()
        .filter(|&(_, s)| *s)
        .map(|(i, _)| i)
        .collect();

    for &idx in &selected_indices {
        let data = read_record_at(&mut r1_file, r1_index[idx])?;
        r1_buf.extend_from_slice(&data);
    }

    // Read selected R2 records
    let r2_buf = if let Some(r2_path) = &tile.r2_path {
        let r2_index = build_record_index(r2_path)?;
        let mut r2_file = BufReader::new(
            File::open(r2_path).map_err(|e| SubsampleError::OpenFile(e, r2_path.clone()))?,
        );
        let mut buf = Vec::new();

        for &idx in &selected_indices {
            if idx < r2_index.len() {
                let data = read_record_at(&mut r2_file, r2_index[idx])?;
                buf.extend_from_slice(&data);
            }
        }

        Some(buf)
    } else {
        None
    };

    Ok((r1_buf, r2_buf))
}

/// Find the next valid FASTQ record after the given byte position.
/// Returns (record_start_position, record_bytes) or None if EOF.
fn find_record_after(
    reader: &mut BufReader<File>,
    pos: u64,
) -> io::Result<Option<(u64, Vec<u8>)>> {
    reader.seek(SeekFrom::Start(pos))?;

    // Skip rest of current (partial) line
    let mut discard = Vec::new();
    if reader.read_until(b'\n', &mut discard)? == 0 {
        return Ok(None);
    }

    // Try up to 4 line offsets to find a record boundary
    for _ in 0..4 {
        let candidate_pos = reader.stream_position()?;

        let mut line1 = Vec::new();
        if reader.read_until(b'\n', &mut line1)? == 0 {
            return Ok(None);
        }

        if line1.starts_with(b"@") {
            let mut line2 = Vec::new();
            let mut line3 = Vec::new();
            let mut line4 = Vec::new();

            if reader.read_until(b'\n', &mut line2)? == 0 {
                return Ok(None);
            }
            if reader.read_until(b'\n', &mut line3)? == 0 {
                return Ok(None);
            }
            if reader.read_until(b'\n', &mut line4)? == 0 {
                // Accept record at EOF without trailing newline
                let mut data = Vec::new();
                data.extend_from_slice(&line1);
                data.extend_from_slice(&line2);
                data.extend_from_slice(&line3);
                data.extend_from_slice(&line4);
                if !data.ends_with(b"\n") {
                    data.push(b'\n');
                }
                return Ok(Some((candidate_pos, data)));
            }

            if line3.starts_with(b"+") {
                let mut data = Vec::new();
                data.extend_from_slice(&line1);
                data.extend_from_slice(&line2);
                data.extend_from_slice(&line3);
                data.extend_from_slice(&line4);
                return Ok(Some((candidate_pos, data)));
            }

            // Not a valid record, seek back and skip this line
            reader.seek(SeekFrom::Start(candidate_pos))?;
            let mut skip = Vec::new();
            reader.read_until(b'\n', &mut skip)?;
        }
    }

    Ok(None)
}

fn exponential_sample<Rng>(rng: &mut Rng, mean: f64) -> f64
where
    Rng: rand::Rng,
{
    -mean * rng.random::<f64>().ln()
}

fn estimate_avg_record_size(path: &Path) -> Result<f64, SubsampleError> {
    let mut reader = BufReader::new(
        File::open(path).map_err(|e| SubsampleError::OpenFile(e, path.into()))?,
    );

    let mut total_bytes: u64 = 0;
    let mut records = 0u64;
    let mut line_buf = Vec::new();
    let max_records = 100;

    while records < max_records {
        let mut record_bytes: u64 = 0;

        for _ in 0..4 {
            line_buf.clear();
            let n = reader.read_until(b'\n', &mut line_buf)?;
            if n == 0 {
                if record_bytes > 0 {
                    return Ok(total_bytes as f64 / records.max(1) as f64);
                }
                return Ok(1.0); // avoid division by zero
            }
            record_bytes += n as u64;
        }

        total_bytes += record_bytes;
        records += 1;
    }

    Ok(total_bytes as f64 / records as f64)
}

fn write_output_data(data: &[u8], dst: &Path, compression_threads: usize) -> io::Result<()> {
    if is_gzipped(dst) && compression_threads > 1 && !data.is_empty() {
        write_output_parallel_gz(data, dst, compression_threads)
    } else if is_gzipped(dst) {
        let file = BufWriter::new(File::create(dst)?);
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(data)?;
        encoder.finish()?;
        Ok(())
    } else {
        let mut file = BufWriter::new(File::create(dst)?);
        file.write_all(data)?;
        file.flush()?;
        Ok(())
    }
}

fn write_output_parallel_gz(data: &[u8], dst: &Path, threads: usize) -> io::Result<()> {
    let chunk_size = (data.len() + threads - 1) / threads;
    let chunks: Vec<&[u8]> = data.chunks(chunk_size).collect();

    let compressed_chunks: Vec<Vec<u8>> = std::thread::scope(|s| {
        let handles: Vec<_> = chunks
            .into_iter()
            .map(|chunk| {
                s.spawn(|| -> io::Result<Vec<u8>> {
                    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                    encoder.write_all(chunk)?;
                    encoder.finish()
                })
            })
            .collect();

        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect::<Result<Vec<_>, _>>()
    })?;

    let mut file = BufWriter::new(File::create(dst)?);
    for chunk in compressed_chunks {
        file.write_all(&chunk)?;
    }
    file.flush()?;

    Ok(())
}

#[derive(Debug, Error)]
pub enum SubsampleError {
    #[error("I/O error")]
    Io(#[from] io::Error),
    #[error("could not open file: {1}")]
    OpenFile(#[source] io::Error, PathBuf),
    #[error("could not create file: {1}")]
    CreateFile(#[source] io::Error, PathBuf),
    #[error("could not create temp directory")]
    TempDir(#[source] io::Error),
    #[error("missing pair source: {0}")]
    MissingSource(&'static str),
    #[error("missing pair destination: {0}")]
    MissingDestination(&'static str),
    #[error("invalid probability: expected (0.0, 1.0), got {0}")]
    InvalidProbability(f64),
    #[error("{0} unexpectedly ended")]
    UnexpectedEof(&'static str),
    #[error("invalid uniform range")]
    InvalidUniformRange(rand::distr::uniform::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subsample_single() -> Result<(), SubsampleError> {
        let data = b"@r1\nACGT\n+\nFQLB
@r2\nACGT\n+\nFQLB
@r3\nACGT\n+\nFQLB
@r4\nACGT\n+\nFQLB
";

        let mut reader = fastq::io::Reader::new(&data[..]);
        let mut writer = fastq::io::Writer::new(Vec::new());

        let mut rng = SmallRng::seed_from_u64(0);

        subsample_single(&mut reader, &mut writer, &mut rng, 0.33)?;

        let expected = b"@r1\nACGT\n+\nFQLB\n@r4\nACGT\n+\nFQLB\n";
        assert_eq!(writer.get_ref(), expected);

        Ok(())
    }

    #[test]
    fn test_subsample_paired() -> Result<(), SubsampleError> {
        let r1_data = b"@r1\nACGT\n+\nFQLB
@r2\nACGT\n+\nFQLB
@r3\nACGT\n+\nFQLB
@r4\nACGT\n+\nFQLB
";

        let r2_data = b"@r1\nTGCA\n+\nBLQF
@r2\nTGCA\n+\nBLQF
@r3\nTGCA\n+\nBLQF
@r4\nTGCA\n+\nBLQF
";

        let mut r1 = fastq::io::Reader::new(&r1_data[..]);
        let mut w1 = fastq::io::Writer::new(Vec::new());
        let mut r2 = fastq::io::Reader::new(&r2_data[..]);
        let mut w2 = fastq::io::Writer::new(Vec::new());

        let mut rng = SmallRng::seed_from_u64(0);

        subsample_paired((&mut r1, &mut w1), (&mut r2, &mut w2), &mut rng, 0.33)?;

        let w1_expected = b"@r1\nACGT\n+\nFQLB\n@r4\nACGT\n+\nFQLB\n";
        assert_eq!(w1.get_ref(), w1_expected);

        let w2_expected = b"@r1\nTGCA\n+\nBLQF\n@r4\nTGCA\n+\nBLQF\n";
        assert_eq!(w2.get_ref(), w2_expected);

        Ok(())
    }

    #[test]
    fn test_subsample_exact_single() -> Result<(), SubsampleError> {
        let data = b"@r1\nACGT\n+\nFQLB
@r2\nACGT\n+\nFQLB
@r3\nACGT\n+\nFQLB
@r4\nACGT\n+\nFQLB
";

        let mut reader = fastq::io::Reader::new(&data[..]);
        let mut writer = fastq::io::Writer::new(Vec::new());

        let bitmap = BitVec::from_element(0b00000011);

        subsample_exact_single(&mut reader, &mut writer, &bitmap)?;

        let expected = b"@r1\nACGT\n+\nFQLB\n@r2\nACGT\n+\nFQLB\n";
        assert_eq!(writer.get_ref(), expected);

        Ok(())
    }

    #[test]
    fn test_subsample_exact_paired() -> Result<(), SubsampleError> {
        let r1_data = b"@r1\nACGT\n+\nFQLB
@r2\nACGT\n+\nFQLB
@r3\nACGT\n+\nFQLB
@r4\nACGT\n+\nFQLB
";

        let r2_data = b"@r1\nTGCA\n+\nBLQF
@r2\nTGCA\n+\nBLQF
@r3\nTGCA\n+\nBLQF
@r4\nTGCA\n+\nBLQF
";

        let mut r1 = fastq::io::Reader::new(&r1_data[..]);
        let mut w1 = fastq::io::Writer::new(Vec::new());
        let mut r2 = fastq::io::Reader::new(&r2_data[..]);
        let mut w2 = fastq::io::Writer::new(Vec::new());

        let bitmap = BitVec::from_element(0b00000011);

        subsample_exact_paired((&mut r1, &mut w1), (&mut r2, &mut w2), &bitmap)?;

        let w1_expected = b"@r1\nACGT\n+\nFQLB\n@r2\nACGT\n+\nFQLB\n";
        assert_eq!(w1.get_ref(), w1_expected);

        let w2_expected = b"@r1\nTGCA\n+\nBLQF\n@r2\nTGCA\n+\nBLQF\n";
        assert_eq!(w2.get_ref(), w2_expected);

        Ok(())
    }

    #[test]
    fn test_parse_tile_bin() {
        let key = parse_tile_bin(b"@A00226:83:HFWFVDSXX:2:1101:1234:5678");
        assert_eq!(key, Some((2u64 << 32) | 1101));

        let key = parse_tile_bin(b"@A00226:83:HFWFVDSXX:1:2205:9876:4321 1:N:0:ACGTACGT");
        assert_eq!(key, Some((1u64 << 32) | 2205));

        let key = parse_tile_bin(b"@INST:100:FC:4:2301:10:20");
        assert_eq!(key, Some((4u64 << 32) | 2301));

        assert_eq!(parse_tile_bin(b"@INST:100:FC"), None);
        assert_eq!(parse_tile_bin(b"@INST:100:FC:X:1101:1:2"), None);
        assert_eq!(parse_tile_bin(b"@INST:100:FC:1:ABC:1:2"), None);

        let key = parse_tile_bin(b"INST:100:FC:3:1201:1:2");
        assert_eq!(key, Some((3u64 << 32) | 1201));
    }

    #[test]
    fn test_exponential_sample() {
        let mut rng = SmallRng::seed_from_u64(42);
        let samples: Vec<f64> = (0..10000).map(|_| exponential_sample(&mut rng, 100.0)).collect();
        let mean: f64 = samples.iter().sum::<f64>() / samples.len() as f64;
        // Mean should be approximately 100
        assert!((mean - 100.0).abs() < 5.0, "mean was {mean}");
        // All samples should be positive
        assert!(samples.iter().all(|&x| x > 0.0));
    }

    #[test]
    fn test_find_record_after() -> io::Result<()> {
        let dir = std::env::temp_dir().join("fq_test_find_record");
        std::fs::create_dir_all(&dir)?;

        let data = b"@r1\nACGT\n+\nFFFF\n@r2\nTGCA\n+\nGGGG\n@r3\nCCCC\n+\nHHHH\n";
        let path = dir.join("test.fq");
        std::fs::write(&path, data)?;

        let mut reader = BufReader::new(File::open(&path)?);

        // find_record_after always skips the rest of the current line first,
        // so seeking to 0 skips "@r1\n" and finds r2
        let result = find_record_after(&mut reader, 0)?;
        assert!(result.is_some());
        let (pos, rec_data) = result.unwrap();
        assert_eq!(pos, 16); // r2 starts at byte 16
        assert!(rec_data.starts_with(b"@r2\n"));

        // Seeking into the middle of r1 also finds r2
        let result = find_record_after(&mut reader, 5)?;
        assert!(result.is_some());
        let (pos, rec_data) = result.unwrap();
        assert_eq!(pos, 16);
        assert!(rec_data.starts_with(b"@r2\n"));

        // Seeking into r2 finds r3
        let result = find_record_after(&mut reader, 20)?;
        assert!(result.is_some());
        let (_, rec_data) = result.unwrap();
        assert!(rec_data.starts_with(b"@r3\n"));

        std::fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn test_build_record_index() -> Result<(), SubsampleError> {
        let dir = std::env::temp_dir().join("fq_test_index");
        std::fs::create_dir_all(&dir).unwrap();

        let data = b"@r1\nACGT\n+\nFFFF\n@r2\nTGCA\n+\nGGGG\n@r3\nCCCC\n+\nHHHH\n";
        let path = dir.join("test.fq");
        std::fs::write(&path, data).unwrap();

        let index = build_record_index(&path)?;
        assert_eq!(index.len(), 3);
        assert_eq!(index[0], 0);
        // Each record is "@rN\nXXXX\n+\nXXXX\n" = 4 + 5 + 2 + 5 = 16 bytes... let me be precise
        // "@r1\n" = 4, "ACGT\n" = 5, "+\n" = 2, "FFFF\n" = 5 → 16
        // But with our record: @r1\nACGT\n+\nFFFF\n = 18 bytes
        // Actually: @r1 = 3 bytes + \n = 4; ACGT + \n = 5; + + \n = 2; FFFF + \n = 5 → total 16
        // Hmm: "@r1\n" is 4 bytes, "ACGT\n" is 5 bytes, "+\n" is 2 bytes, "FFFF\n" is 5 bytes = 16
        // Wait: @ r 1 \n = 4, A C G T \n = 5, + \n = 2, F F F F \n = 5 => 16
        // Then @r2 starts at offset 16
        assert_eq!(index[1], 16);
        assert_eq!(index[2], 32);

        std::fs::remove_dir_all(&dir).unwrap();

        Ok(())
    }

    #[test]
    fn test_subsample_by_tile_exact_single() -> Result<(), SubsampleError> {
        let dir = std::env::temp_dir().join("fq_test_tile_exact_single");
        std::fs::create_dir_all(&dir).unwrap();

        let r1_data = b"\
@A:1:FC:1:1101:10:20\nACGT\n+\nFFFF
@A:1:FC:1:1101:30:40\nTGCA\n+\nFFFF
@A:1:FC:1:1102:50:60\nGCTA\n+\nFFFF
@A:1:FC:1:1101:70:80\nCGAT\n+\nFFFF
";

        let r1_src = dir.join("r1.fq");
        let r1_dst = dir.join("r1_out.fq");
        std::fs::write(&r1_src, r1_data).unwrap();

        let rng = SmallRng::seed_from_u64(42);
        subsample_by_tile(
            (&r1_src, &r1_dst),
            (None, None),
            rng,
            TileCountMode::Explicit(2),
            false, // not fast (use exact)
            1,
            1,
            false, // not in-memory
            None,  // default temp dir
        )?;

        let output = std::fs::read_to_string(&r1_dst).unwrap();
        let output_records: Vec<&str> = output.trim().split('\n').collect();
        assert_eq!(output_records.len(), 8, "expected 2 records (8 lines), got: {output}");

        for line in output_records.iter().step_by(4) {
            assert!(line.contains(":1101:"), "expected tile 1101, got: {line}");
        }

        std::fs::remove_dir_all(&dir).unwrap();
        Ok(())
    }

    #[test]
    fn test_subsample_by_tile_skip_ahead_single() -> Result<(), SubsampleError> {
        let dir = std::env::temp_dir().join("fq_test_tile_skip_single");
        std::fs::create_dir_all(&dir).unwrap();

        // 10 records on tile 1101, 1 on tile 1102. Target 5 per tile.
        // Tile 1101 retained (10 >= 5), tile 1102 discarded (1 < 5).
        let mut data = Vec::new();
        for i in 0..10 {
            data.extend_from_slice(
                format!("@A:1:FC:1:1101:{}:20\nACGT\n+\nFFFF\n", i * 10).as_bytes(),
            );
        }
        data.extend_from_slice(b"@A:1:FC:1:1102:50:60\nGCTA\n+\nFFFF\n");

        let r1_src = dir.join("r1.fq");
        let r1_dst = dir.join("r1_out.fq");
        std::fs::write(&r1_src, &data).unwrap();

        let rng = SmallRng::seed_from_u64(42);
        subsample_by_tile(
            (&r1_src, &r1_dst),
            (None, None),
            rng,
            TileCountMode::Explicit(5),
            true, // fast (skip-ahead)
            1,
            1,
            false,
            None,
        )?;

        let output = std::fs::read_to_string(&r1_dst).unwrap();
        let record_count = output.trim().split('\n').count() / 4;
        // Skip-ahead is approximate, so we check it's in a reasonable range
        assert!(
            record_count >= 3 && record_count <= 7,
            "expected ~5 records, got {record_count}"
        );

        // All records should be from tile 1101
        for line in output.trim().split('\n').step_by(4) {
            assert!(line.contains(":1101:"), "expected tile 1101, got: {line}");
        }

        std::fs::remove_dir_all(&dir).unwrap();
        Ok(())
    }

    #[test]
    fn test_subsample_by_tile_exact_paired() -> Result<(), SubsampleError> {
        let dir = std::env::temp_dir().join("fq_test_tile_exact_paired");
        std::fs::create_dir_all(&dir).unwrap();

        let r1_data = b"\
@A:1:FC:1:1101:10:20\nACGT\n+\nFFFF
@A:1:FC:1:1101:30:40\nTGCA\n+\nFFFF
@A:1:FC:1:1102:50:60\nGCTA\n+\nFFFF
@A:1:FC:1:1101:70:80\nCGAT\n+\nFFFF
";
        let r2_data = b"\
@A:1:FC:1:1101:10:20\nAAAA\n+\nFFFF
@A:1:FC:1:1101:30:40\nCCCC\n+\nFFFF
@A:1:FC:1:1102:50:60\nGGGG\n+\nFFFF
@A:1:FC:1:1101:70:80\nTTTT\n+\nFFFF
";

        let r1_src = dir.join("r1.fq");
        let r1_dst = dir.join("r1_out.fq");
        let r2_src = dir.join("r2.fq");
        let r2_dst = dir.join("r2_out.fq");

        std::fs::write(&r1_src, r1_data).unwrap();
        std::fs::write(&r2_src, r2_data).unwrap();

        let rng = SmallRng::seed_from_u64(42);
        subsample_by_tile(
            (&r1_src, &r1_dst),
            (Some(r2_src.as_path()), Some(r2_dst.as_path())),
            rng,
            TileCountMode::Explicit(2),
            false, // not fast (use exact)
            1,
            1,
            false,
            None,
        )?;

        let r1_output = std::fs::read_to_string(&r1_dst).unwrap();
        let r2_output = std::fs::read_to_string(&r2_dst).unwrap();

        let r1_lines: Vec<&str> = r1_output.trim().split('\n').collect();
        let r2_lines: Vec<&str> = r2_output.trim().split('\n').collect();

        assert_eq!(r1_lines.len(), 8);
        assert_eq!(r2_lines.len(), 8);

        // R1 and R2 names should match
        for i in (0..r1_lines.len()).step_by(4) {
            let r1_name = r1_lines[i].split(' ').next().unwrap();
            let r2_name = r2_lines[i].split(' ').next().unwrap();
            assert_eq!(r1_name, r2_name);
        }

        std::fs::remove_dir_all(&dir).unwrap();
        Ok(())
    }

    #[test]
    fn test_subsample_by_tile_from_record_count() -> Result<(), SubsampleError> {
        let dir = std::env::temp_dir().join("fq_test_tile_from_count2");
        std::fs::create_dir_all(&dir).unwrap();

        let data = b"\
@A:1:FC:1:1101:10:20\nACGT\n+\nFFFF
@A:1:FC:1:1102:10:20\nTGCA\n+\nFFFF
@A:1:FC:1:1101:30:40\nGCTA\n+\nFFFF
@A:1:FC:1:1102:30:40\nCGAT\n+\nFFFF
@A:1:FC:1:1101:50:60\nACGT\n+\nFFFF
@A:1:FC:1:1102:50:60\nTGCA\n+\nFFFF
";

        let r1_src = dir.join("r1.fq");
        let r1_dst = dir.join("r1_out.fq");
        std::fs::write(&r1_src, data).unwrap();

        let rng = SmallRng::seed_from_u64(42);
        subsample_by_tile(
            (&r1_src, &r1_dst),
            (None, None),
            rng,
            TileCountMode::FromRecordCount(4),
            false, // not fast (use exact)
            1,
            1,
            false,
            None,
        )?;

        let output = std::fs::read_to_string(&r1_dst).unwrap();
        let output_lines: Vec<&str> = output.trim().split('\n').collect();
        assert_eq!(output_lines.len(), 16, "expected 4 records (16 lines), got: {output}");

        std::fs::remove_dir_all(&dir).unwrap();
        Ok(())
    }

    #[test]
    fn test_subsample_by_tile_in_memory() -> Result<(), SubsampleError> {
        let dir = std::env::temp_dir().join("fq_test_tile_inmem");
        std::fs::create_dir_all(&dir).unwrap();

        let r1_data = b"\
@A:1:FC:1:1101:10:20\nACGT\n+\nFFFF
@A:1:FC:1:1101:30:40\nTGCA\n+\nFFFF
@A:1:FC:1:1102:50:60\nGCTA\n+\nFFFF
@A:1:FC:1:1101:70:80\nCGAT\n+\nFFFF
";
        let r2_data = b"\
@A:1:FC:1:1101:10:20\nAAAA\n+\nFFFF
@A:1:FC:1:1101:30:40\nCCCC\n+\nFFFF
@A:1:FC:1:1102:50:60\nGGGG\n+\nFFFF
@A:1:FC:1:1101:70:80\nTTTT\n+\nFFFF
";

        let r1_src = dir.join("r1.fq");
        let r1_dst = dir.join("r1_out.fq");
        let r2_src = dir.join("r2.fq");
        let r2_dst = dir.join("r2_out.fq");

        std::fs::write(&r1_src, r1_data).unwrap();
        std::fs::write(&r2_src, r2_data).unwrap();

        let rng = SmallRng::seed_from_u64(42);
        subsample_by_tile(
            (&r1_src, &r1_dst),
            (Some(r2_src.as_path()), Some(r2_dst.as_path())),
            rng,
            TileCountMode::Explicit(2),
            false, // not fast; doesn't matter for in-memory
            1,
            1,
            true, // in-memory
            None,
        )?;

        let r1_output = std::fs::read_to_string(&r1_dst).unwrap();
        let r2_output = std::fs::read_to_string(&r2_dst).unwrap();

        let r1_lines: Vec<&str> = r1_output.trim().split('\n').collect();
        let r2_lines: Vec<&str> = r2_output.trim().split('\n').collect();

        // 2 records from tile 1101 (3 records, 2 selected), tile 1102 discarded (1 < 2)
        assert_eq!(r1_lines.len(), 8, "expected 2 R1 records, got: {r1_output}");
        assert_eq!(r2_lines.len(), 8, "expected 2 R2 records, got: {r2_output}");

        // All R1 records from tile 1101
        for line in r1_lines.iter().step_by(4) {
            assert!(line.contains(":1101:"), "expected tile 1101, got: {line}");
        }

        std::fs::remove_dir_all(&dir).unwrap();
        Ok(())
    }

    #[test]
    fn test_subsample_by_tile_all_bins_too_small() -> Result<(), SubsampleError> {
        let dir = std::env::temp_dir().join("fq_test_tile_empty2");
        std::fs::create_dir_all(&dir).unwrap();

        let data = b"\
@A:1:FC:1:1101:10:20\nACGT\n+\nFFFF
@A:1:FC:1:1101:30:40\nTGCA\n+\nFFFF
@A:1:FC:1:1102:50:60\nGCTA\n+\nFFFF
@A:1:FC:1:1102:70:80\nCGAT\n+\nFFFF
";

        let r1_src = dir.join("r1.fq");
        let r1_dst = dir.join("r1_out.fq");
        std::fs::write(&r1_src, data).unwrap();

        let rng = SmallRng::seed_from_u64(42);
        subsample_by_tile((&r1_src, &r1_dst), (None, None), rng, TileCountMode::Explicit(3), false, 1, 1, false, None)?;

        let output = std::fs::read_to_string(&r1_dst).unwrap();
        assert!(output.is_empty(), "expected empty output, got: {output}");

        std::fs::remove_dir_all(&dir).unwrap();
        Ok(())
    }

    #[test]
    fn test_count_records_from_index_found() -> io::Result<()> {
        let dir = std::env::temp_dir().join("fq_test_index_count2");
        std::fs::create_dir_all(&dir)?;

        let fq_path = dir.join("test.fq");
        let fai_path = dir.join("test.fq.fai");

        std::fs::write(&fq_path, b"@r1\nACGT\n+\nFFFF\n@r2\nTGCA\n+\nFFFF\n@r3\nGGGG\n+\nFFFF\n@r4\nCCCC\n+\nFFFF\n")?;
        std::fs::write(&fai_path, b"r1\t4\t4\t4\t5\t14\nr2\t4\t24\t4\t5\t34\nr3\t4\t44\t4\t5\t54\nr4\t4\t64\t4\t5\t74\n")?;

        let result = count_records_from_index(&fq_path)?;
        assert_eq!(result, Some(4));

        std::fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn test_count_records_from_index_missing() -> io::Result<()> {
        let dir = std::env::temp_dir().join("fq_test_index_missing2");
        std::fs::create_dir_all(&dir)?;

        let fq_path = dir.join("test.fq");
        std::fs::write(&fq_path, b"@r1\nACGT\n+\nFFFF\n")?;

        let result = count_records_from_index(&fq_path)?;
        assert_eq!(result, None);

        std::fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn test_count_records_from_index_gz() -> io::Result<()> {
        let dir = std::env::temp_dir().join("fq_test_index_gz2");
        std::fs::create_dir_all(&dir)?;

        let fq_path = dir.join("test.fq.gz");
        let fai_path = dir.join("test.fq.gz.fai");

        std::fs::write(&fq_path, b"dummy")?;
        std::fs::write(&fai_path, b"r1\t4\t4\t4\t5\t14\nr2\t4\t24\t4\t5\t34\nr3\t4\t44\t4\t5\t54\n")?;

        let result = count_records_from_index(&fq_path)?;
        assert_eq!(result, Some(3));

        std::fs::remove_dir_all(&dir)?;
        Ok(())
    }
}
