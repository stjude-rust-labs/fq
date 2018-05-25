#[derive(Debug, Default, Eq, PartialEq)]
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

    pub fn clear(&mut self) {
        self.name.clear();
        self.sequence.clear();
        self.quality.clear();
    }

    pub fn reset(&mut self) {
        if let Some(i) = self.name.rfind('/') {
            self.name.truncate(i);
        }
    }
}
