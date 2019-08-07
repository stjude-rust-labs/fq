pub use self::filter::filter;
pub use self::generate::generate;
pub use self::lint::lint;

pub mod filter;
pub mod generate;
pub mod lint;

use std::{io, process};

fn exit_with_io_error(error: &io::Error, pathname: Option<&str>) -> ! {
    match pathname {
        Some(p) => eprintln!("{}: {}", error, p),
        None => eprintln!("{}", error),
    }

    process::exit(1);
}
