#[macro_use] extern crate clap;
extern crate fqlib;

use clap::{App, Arg};
use fqlib::PairedFastQReader;
use fqlib::validators::validate_pair;

fn main() {
    let matches = App::new("fqlint")
        .version(crate_version!())
        .arg(Arg::with_name("lint-mode")
             .long("lint-mode")
             .value_name("MODE")
             .possible_values(&["minimum", "low", "high"])
             .default_value("high"))
        .arg(Arg::with_name("r1-input-pathname")
             .help("")
             .index(1)
             .required(true))
        .arg(Arg::with_name("r2-input-pathname")
             .help("")
             .index(2)
             .required(true))
        .get_matches();

    let r1_input_pathname = matches.value_of("r1-input-pathname").unwrap();
    let r2_input_pathname = matches.value_of("r2-input-pathname").unwrap();

    let reader = PairedFastQReader::open(
        r1_input_pathname,
        r2_input_pathname,
    ).unwrap();

    for (r1_block, r2_block) in reader {
        let b = r1_block.unwrap();
        let d = r2_block.unwrap();

        validate_pair(&b, &d).unwrap();
    }
}
