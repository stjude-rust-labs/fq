#[macro_use] extern crate log;
extern crate env_logger;
#[macro_use] extern crate clap;
extern crate fqlib;

use std::io;
use std::process;

use clap::{App, Arg};
use fqlib::{BlockPairGenerator, PairedWriter, writers};
use log::LevelFilter;

fn exit_with_io_error(error: io::Error, pathname: Option<&str>) -> ! {
    match pathname {
        Some(p) => eprintln!("{}: {}", error, p),
        None => eprintln!("{}", error),
    }

    process::exit(1);
}

fn main() {
    let matches = App::new("fqgen")
        .version(crate_version!())
        .arg(Arg::with_name("num-blocks")
             .short("n")
             .long("num-blocks")
             .help("Number of blocks to generate")
             .value_name("N")
             .default_value("10000"))
        .arg(Arg::with_name("verbose")
             .short("v")
             .long("verbose")
             .help("Use verbose logging"))
        .arg(Arg::with_name("r1-output-pathname")
             .index(1)
             .required(true))
        .arg(Arg::with_name("r2-output-pathname")
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

    let num_blocks = value_t!(matches, "num-blocks", i32).unwrap_or_else(|e| e.exit());

    info!("fqgen start");

    let generator = BlockPairGenerator::new();

    let w1 = match writers::factory(r1_output_pathname) {
        Ok(w) => w,
        Err(e) => exit_with_io_error(e, Some(r1_output_pathname)),
    };

    let w2 = match writers::factory(r2_output_pathname) {
        Ok(w) => w,
        Err(e) => exit_with_io_error(e, Some(r2_output_pathname)),
    };

    let mut writer = PairedWriter::new(w1, w2);

    if let Err(e) = writer.write(generator, num_blocks) {
        exit_with_io_error(e, None);
    }

    info!("fqgen end");
}
