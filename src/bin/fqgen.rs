#[macro_use] extern crate log;
extern crate env_logger;
#[macro_use] extern crate clap;
extern crate flate2;
extern crate fqlib;

use std::io::BufWriter;
use std::fs::File;

use clap::{App, Arg};
use flate2::write::GzEncoder;
use fqlib::{BlockPairGenerator, Writer};
use log::LevelFilter;

fn main() {
    let matches = App::new("fqgen")
        .version(crate_version!())
        .arg(Arg::with_name("compress")
             .help("Compress output with gzip")
             .short("c")
             .long("compress"))
        .arg(Arg::with_name("num-reads")
             .short("n")
             .long("num-reads")
             .value_name("N")
             .default_value("10000"))
        .arg(Arg::with_name("verbose")
             .short("v")
             .long("verbose")
             .help("Use verbose logging"))
        .arg(Arg::with_name("r1-output-pathname")
             .help("FIXME")
             .index(1)
             .required(true))
        .arg(Arg::with_name("r2-output-pathname")
             .help("FIXME")
             .index(2)
             .required(true))
        .get_matches();

    if matches.is_present("verbose") {
        env_logger::Builder::from_default_env()
            .filter(Some("fqgen"), LevelFilter::Info)
            .filter(Some(crate_name!()), LevelFilter::Info)
            .init();
    }

    let r1_output_pathname = matches.value_of("r1-output-pathname").unwrap();
    let r2_output_pathname = matches.value_of("r2-output-pathname").unwrap();

    let num_reads = value_t!(matches, "num-reads", i32).unwrap_or_else(|e| e.exit());
    let compress = matches.is_present("compress");

    info!("fqgen start");

    let generator = BlockPairGenerator::new();

    if compress {
        let mut writer = Writer::<GzEncoder<BufWriter<File>>>::gz_create(
            &r1_output_pathname,
            &r2_output_pathname,
        ).unwrap();

        writer.write(generator, num_reads).unwrap();
    } else {
        let mut writer = Writer::<BufWriter<File>>::create(
            &r1_output_pathname,
            &r2_output_pathname,
        ).unwrap();

        writer.write(generator, num_reads).unwrap();
    }

    info!("fqgen end");
}
