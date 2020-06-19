use clap::{value_t, ArgMatches};
use log::info;
use noodles_fastq as fastq;

use crate::{Generator, PairWriter};

use super::exit_with_io_error;

pub fn generate(matches: &ArgMatches) {
    let r1_dst = matches.value_of("r1-dst").unwrap();
    let r2_dst = matches.value_of("r2-dst").unwrap();

    let n_records = value_t!(matches, "n-records", i32).unwrap_or_else(|e| e.exit());

    info!("fq-generate start");

    let generator = if matches.is_present("seed") {
        let seed = value_t!(matches, "seed", u64).unwrap_or_else(|e| e.exit());
        Generator::seed_from_u64(seed)
    } else {
        Generator::new()
    };

    let w1 = fastq::writer::create(r1_dst).unwrap_or_else(|e| exit_with_io_error(&e, Some(r1_dst)));
    let w2 = fastq::writer::create(r2_dst).unwrap_or_else(|e| exit_with_io_error(&e, Some(r2_dst)));
    let mut writer = PairWriter::new(w1, w2);

    writer
        .write(generator, n_records)
        .unwrap_or_else(|e| exit_with_io_error(&e, None));

    info!("fq-generate end");
}
