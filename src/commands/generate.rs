use std::io;
use std::process;

use clap::ArgMatches;
use noodles::formats::fastq;

use crate::{Generator, PairWriter};

fn exit_with_io_error(error: io::Error, pathname: Option<&str>) -> ! {
    match pathname {
        Some(p) => eprintln!("{}: {}", error, p),
        None => eprintln!("{}", error),
    }

    process::exit(1);
}

pub fn generate(matches: &ArgMatches) {
    let r1_output_pathname = matches.value_of("out1").unwrap();
    let r2_output_pathname = matches.value_of("out2").unwrap();

    let num_blocks = value_t!(matches, "num-blocks", i32).unwrap_or_else(|e| e.exit());

    info!("fq-generate start");

    let generator = Generator::new();

    let w1 = match fastq::writer::create(r1_output_pathname) {
        Ok(w) => w,
        Err(e) => exit_with_io_error(e, Some(r1_output_pathname)),
    };

    let w2 = match fastq::writer::create(r2_output_pathname) {
        Ok(w) => w,
        Err(e) => exit_with_io_error(e, Some(r2_output_pathname)),
    };

    let mut writer = PairWriter::new(w1, w2);

    if let Err(e) = writer.write(generator, num_blocks) {
        exit_with_io_error(e, None);
    }

    info!("fq-generate end");
}
