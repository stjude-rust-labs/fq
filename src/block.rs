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
    /// If `name` includes a trailing interleave, it is removed.
    ///
    /// # Examples
    ///
    /// ```
    /// use fqlib::Block;
    ///
    /// let block = Block::new("@fqlib/1", "AGCT", "+", "abcd");
    ///
    /// assert_eq!(block.name, "@fqlib");
    /// assert_eq!(block.sequence, "AGCT");
    /// assert_eq!(block.plus_line, "+");
    /// assert_eq!(block.quality, "abcd");
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

    /// Truncates all line buffers to 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use fqlib::Block;
    ///
    /// let mut block = Block::new("@fqlib/1", "AGCT", "+", "abcd");
    /// block.clear();
    ///
    /// assert!(block.name.is_empty());
    /// assert!(block.sequence.is_empty());
    /// assert!(block.plus_line.is_empty());
    /// assert!(block.quality.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.name.clear();
        self.sequence.clear();
        self.plus_line.clear();
        self.quality.clear();
    }

    pub fn reset(&mut self) {
        if let Some(i) = self.name.rfind('/') {
            self.name.truncate(i);
        }
    }
}
