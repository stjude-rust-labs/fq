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
    pub fn new<S>(name: S, sequence: S, plus_line: S, quality: S) -> Block
    where
        S: Into<String>,
    {
        Block {
            name: name.into(),
            sequence: sequence.into(),
            plus_line: plus_line.into(),
            quality: quality.into(),
        }
    }
}
