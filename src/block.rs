#[derive(Debug, Eq, PartialEq)]
pub struct Block {
    pub name: String,
    pub sequence: String,
    pub plus_line: String,
    pub quality: String,
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
        let mut block = Block {
            name: name.into(),
            sequence: sequence.into(),
            plus_line: plus_line.into(),
            quality: quality.into(),
        };

        block.reset();

        block
    }

    pub fn reset(&mut self) {
        if let Some(i) = self.name.rfind('/') {
            self.name.truncate(i);
        }
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
