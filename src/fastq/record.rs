use bytes::Bytes;

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct Record {
    pub buf: Bytes,
    pub definition_end: usize,
    pub name_end: usize,
    pub sequence_end: usize,
    pub plus_line_end: usize,
}

const LINE_FEED: u8 = b'\n';
const CARRIAGE_RETURN: u8 = b'\r';

impl Record {
    pub fn new<S, T, U, V>(name: S, sequence: T, plus_line: U, quality_scores: V) -> Self
    where
        S: AsRef<[u8]>,
        T: AsRef<[u8]>,
        U: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        let mut buf = Vec::new();

        buf.extend(name.as_ref());
        buf.push(LINE_FEED);
        let definition_end = buf.len();

        buf.extend(sequence.as_ref());
        buf.push(LINE_FEED);
        let sequence_end = buf.len();

        buf.extend(plus_line.as_ref());
        buf.push(LINE_FEED);
        let plus_line_end = buf.len();

        buf.extend(quality_scores.as_ref());
        buf.push(LINE_FEED);

        Self {
            buf: Bytes::from(buf),
            definition_end,
            name_end: definition_end,
            sequence_end,
            plus_line_end,
        }
    }

    fn definition(&self) -> &[u8] {
        trim_newline_end(&self.buf[0..self.definition_end])
    }

    pub fn name(&self) -> &[u8] {
        trim_newline_end(&self.buf[0..self.name_end])
    }

    pub fn sequence(&self) -> &[u8] {
        trim_newline_end(&self.buf[self.definition_end..self.sequence_end])
    }

    pub fn plus_line(&self) -> &[u8] {
        trim_newline_end(&self.buf[self.sequence_end..self.plus_line_end])
    }

    pub fn quality_scores(&self) -> &[u8] {
        trim_newline_end(&self.buf[self.plus_line_end..])
    }

    pub fn clear(&mut self) {
        self.buf.clear();
        self.definition_end = 0;
        self.name_end = 0;
        self.sequence_end = 0;
        self.plus_line_end = 0;
    }

    /// Removes the description from the name.
    pub fn reset(&mut self, separator: Option<u8>) {
        let definition = self.definition();

        let pos = if let Some(c) = separator {
            definition.iter().position(|&b| b == c)
        } else {
            definition.iter().position(|&b| b == b'/' || b == b' ')
        };

        if let Some(i) = pos {
            self.name_end = i;
        }
    }
}

impl AsRef<[u8]> for Record {
    fn as_ref(&self) -> &[u8] {
        &self.buf
    }
}

fn trim_newline_end(mut buf: &[u8]) -> &[u8] {
    if buf.ends_with(&[LINE_FEED]) {
        buf = &buf[..buf.len() - 1];

        if buf.ends_with(&[CARRIAGE_RETURN]) {
            buf = &buf[..buf.len() - 1];
        }
    }

    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear() {
        let mut record = Record::new("@fqlib:1/1", "ACGT", "+", "FQLB");

        record.clear();

        assert!(record.name().is_empty());
        assert!(record.sequence().is_empty());
        assert!(record.plus_line().is_empty());
        assert!(record.quality_scores().is_empty());
    }

    #[test]
    fn test_reset() {
        fn t(definition: &str, separator: Option<u8>, expected: &[u8]) {
            let mut record = Record::new(definition, "", "", "");
            record.reset(separator);
            assert_eq!(record.name(), expected);
        }

        t("@fqlib/1", None, b"@fqlib");
        t("@fqlib 1", None, b"@fqlib");
        t("@fqlib/1 RG:rg0", None, b"@fqlib");
        t("@fqlib_1", Some(b'_'), b"@fqlib");
    }
}
