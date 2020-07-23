#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct Record {
    name: Vec<u8>,
    sequence: Vec<u8>,
    plus_line: Vec<u8>,
    quality_scores: Vec<u8>,
}

impl Record {
    pub fn new<S, T, U, V>(name: S, sequence: T, plus_line: U, quality_scores: V) -> Self
    where
        S: Into<Vec<u8>>,
        T: Into<Vec<u8>>,
        U: Into<Vec<u8>>,
        V: Into<Vec<u8>>,
    {
        Self {
            name: name.into(),
            sequence: sequence.into(),
            plus_line: plus_line.into(),
            quality_scores: quality_scores.into(),
        }
    }

    pub fn name(&self) -> &[u8] {
        &self.name
    }

    pub fn name_mut(&mut self) -> &mut Vec<u8> {
        &mut self.name
    }

    pub fn sequence(&self) -> &[u8] {
        &self.sequence
    }

    pub fn sequence_mut(&mut self) -> &mut Vec<u8> {
        &mut self.sequence
    }

    pub fn plus_line(&self) -> &[u8] {
        &self.plus_line
    }

    pub fn plus_line_mut(&mut self) -> &mut Vec<u8> {
        &mut self.plus_line
    }

    pub fn quality_scores(&self) -> &[u8] {
        &self.quality_scores
    }

    pub fn quality_scores_mut(&mut self) -> &mut Vec<u8> {
        &mut self.quality_scores
    }

    pub fn clear(&mut self) {
        self.name.clear();
        self.sequence.clear();
        self.plus_line.clear();
        self.quality_scores.clear();
    }
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
}
