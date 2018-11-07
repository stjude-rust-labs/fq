use noodles::formats::fastq::Record;

/// Prepares a record after initialization.
///
/// This should be called after clearing and directly writing to the line
/// buffers.
///
/// Resetting only includes removing the interleave or meta from the name, if
/// either is present.
///
/// # Examples
///
/// ```
/// # extern crate fqlib;
/// # extern crate noodles;
/// #
/// # use noodles::formats::fastq::Record;
/// #
/// # fn main() {
/// use fqlib::record;
///
/// let mut r = Record::default();
/// r.name_mut().extend_from_slice(b"@fqlib/1");
/// assert_eq!(r.name(), b"@fqlib/1");
/// record::reset(&mut r);
/// assert_eq!(r.name(), b"@fqlib");
///
/// let mut r = Record::default();
/// r.name_mut().extend_from_slice(b"@fqlib 1");
/// assert_eq!(r.name(), b"@fqlib 1");
/// record::reset(&mut r);
/// assert_eq!(r.name(), b"@fqlib");
/// # }
/// ```
pub fn reset(record: &mut Record) {
    let pos = record.name().iter().rev().position(|&b| b == b'/' || b == b' ');

    if let Some(i) = pos {
        let len = record.name().len();
        record.name_mut().truncate(len - i - 1);
    }
}
