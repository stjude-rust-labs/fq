#[derive(Debug, Eq, PartialEq)]
pub struct Block {
    pub name: String,
    pub sequence: String,
    pub plus_line: String,
    pub quality: String,
    pub interleave: Option<String>,
}

impl Block {
    /// Creates a new FastQ block.
    ///
    /// # Examples
    ///
    /// ```
    /// use fqlib::Block;
    /// let _ = Block::new("@fqlib/1", "AGCT", "+", "abcd");
    /// ```
    pub fn new<S, T, U, V>(
        name: S,
        sequence: T,
        plus_line: U,
        quality: V,
    ) -> Block
    where
        S: Into<String>,
        T: Into<String>,
        U: Into<String>,
        V: Into<String>,
    {
        let (name, interleave) = parse_name(&name.into());

        Block {
            name: name.into(),
            sequence: sequence.into(),
            plus_line: plus_line.into(),
            quality: quality.into(),
            interleave,
        }
    }
}

fn parse_name(name: &str) -> (String, Option<String>) {
    let pieces: Vec<&str> = name.rsplitn(2, '/').collect();

    if pieces.len() == 2 {
        (pieces[1].to_string(), Some(pieces[0].to_string()))
    } else {
        (pieces[0].to_string(), None)
    }
}

pub struct BlockBuf {
    pub name: String,
    pub sequence: String,
    pub plus_line: String,
    pub quality: String,
}

impl BlockBuf {
    pub fn new() -> BlockBuf {
        BlockBuf {
            name: String::new(),
            sequence: String::new(),
            plus_line: String::new(),
            quality: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_name() {
        let (name, interleave) = parse_name("@fqlib/1");
        assert_eq!(name, "@fqlib");
        assert_eq!(interleave, Some(String::from("1")));

        let (name, interleave) = parse_name("@fqlib");
        assert_eq!(name, "@fqlib");
        assert_eq!(interleave, None);
    }
}
