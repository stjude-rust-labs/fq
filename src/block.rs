/// A FASTQ entry (block) containing a name, sequence, plus line, and quality.
///
/// Note that lines are stored as byte buffers, not strings.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct Block {
    pub name: Vec<u8>,
    pub sequence: Vec<u8>,
    pub plus_line: Vec<u8>,
    pub quality: Vec<u8>,
}

impl Block {
    /// Creates a new FastQ block with the given lines.
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
    /// assert_eq!(block.name(), b"@fqlib");
    /// assert_eq!(block.sequence(), b"AGCT");
    /// assert_eq!(block.plus_line(), b"+");
    /// assert_eq!(block.quality(), b"abcd");
    /// ```
    pub fn new<S, T, U, V>(
        name: S,
        sequence: T,
        plus_line: U,
        quality: V,
    ) -> Block
    where
        S: Into<Vec<u8>>,
        T: Into<Vec<u8>>,
        U: Into<Vec<u8>>,
        V: Into<Vec<u8>>,
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
    /// assert!(block.name().is_empty());
    /// assert!(block.sequence().is_empty());
    /// assert!(block.plus_line().is_empty());
    /// assert!(block.quality().is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.name.clear();
        self.sequence.clear();
        self.plus_line.clear();
        self.quality.clear();
    }

    /// Prepares a block after initialization.
    ///
    /// This should be called after clearing and directly writing to the line buffers.
    ///
    /// Resetting only includes removing the interleave from the name, if one is present.
    ///
    /// # Examples
    ///
    /// ```
    /// use fqlib::Block;
    ///
    /// let mut block = Block::default();
    /// block.name.extend_from_slice(b"@fqlib/1");
    /// assert_eq!(block.name(), b"@fqlib/1");
    /// block.reset();
    /// assert_eq!(block.name(), b"@fqlib");
    ///
    /// let mut block = Block::default();
    /// block.name.extend_from_slice(b"@fqlib 1");
    /// assert_eq!(block.name(), b"@fqlib 1");
    /// block.reset();
    /// assert_eq!(block.name(), b"@fqlib");
    /// ```
    pub fn reset(&mut self) {
        let pos = self.name.iter().rev().enumerate().position(|(_, &j)| j == b'/' || j == b' ');

        if let Some(i) = pos {
            let len = self.name.len();
            self.name.truncate(len - i - 1);
        }
    }

    pub fn name(&self) -> &[u8] {
        &self.name
    }

    pub fn sequence(&self) -> &[u8] {
        &self.sequence
    }

    pub fn plus_line(&self) -> &[u8] {
        &self.plus_line
    }

    pub fn quality(&self) -> &[u8] {
        &self.quality
    }
}
