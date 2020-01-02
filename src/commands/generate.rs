use clap::{value_t, ArgMatches};
use log::info;
use noodles_fastq as fastq;

use crate::{Generator, PairWriter};

use super::exit_with_io_error;

pub fn generate(matches: &ArgMatches) {
    let r1_output_pathname = matches.value_of("out1").unwrap();
    let r2_output_pathname = matches.value_of("out2").unwrap();

    let n_records = value_t!(matches, "n-records", i32).unwrap_or_else(|e| e.exit());

    info!("fq-generate start");

    let generator = Generator::new();

    let w1 = fastq::writer::create(r1_output_pathname)
        .unwrap_or_else(|e| exit_with_io_error(&e, Some(r1_output_pathname)));

    let w2 = fastq::writer::create(r2_output_pathname)
        .unwrap_or_else(|e| exit_with_io_error(&e, Some(r2_output_pathname)));

    let mut writer = PairWriter::new(w1, w2);

    writer
        .write(generator, n_records)
        .unwrap_or_else(|e| exit_with_io_error(&e, None));

    info!("fq-generate end");
}
