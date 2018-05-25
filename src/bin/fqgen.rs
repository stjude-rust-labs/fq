#[macro_use] extern crate clap;
extern crate fqlib;

use std::io::BufWriter;
use std::fs::File;

use clap::{App, Arg};
use fqlib::{BlockPairGenerator, Writer};

fn main() {
    let matches = App::new("fqgen")
        .version(crate_version!())
        .arg(Arg::with_name("num-reads")
             .short("n")
             .long("num-reads")
             .value_name("N")
             .default_value("10000"))
        .arg(Arg::with_name("r1-output-pathname")
             .help("FIXME")
             .index(1)
             .required(true))
        .arg(Arg::with_name("r2-output-pathname")
             .help("FIXME")
             .index(2)
             .required(true))
        .get_matches();

    let r1_output_pathname = matches.value_of("r1-output-pathname").unwrap();
    let r2_output_pathname = matches.value_of("r2-output-pathname").unwrap();

    let num_reads = value_t!(matches, "num-reads", i32).unwrap_or_else(|e| e.exit());

    let generator = BlockPairGenerator::new();
    let mut writer = Writer::<BufWriter<File>>::create(
        &r1_output_pathname,
        &r2_output_pathname,
    ).unwrap();
    writer.write(generator, num_reads).unwrap();
}
