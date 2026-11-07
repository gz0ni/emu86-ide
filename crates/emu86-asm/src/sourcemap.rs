//! Mapping between machine code addresses and source lines.

/// A position in an assembly source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    /// Absolute path to the source file.
    pub file: String,
    /// 1-based line number.
    pub line: u32,
    /// 1-based column number.
    pub column: u32,
}

impl SourceLocation {
    /// Creates a location.
    #[must_use]
    pub fn new(file: impl Into<String>, line: u32, column: u32) -> Self {
        Self {
            file: file.into(),
            line,
            column,
        }
    }
}

/// Sorted list of `(offset -> source location)` pairs built from the
/// assembler listing.
///
/// Offsets are relative to the start of the produced image, which is what the
/// DAP layer needs to turn `CS:IP` into a file and a line.
#[derive(Debug, Clone, Default)]
pub struct SourceMap {
    entries: Vec<(u32, SourceLocation)>,
}

impl SourceMap {
    /// Creates an empty map.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a mapping. Entries do not have to be inserted in order.
    pub fn insert(&mut self, offset: u32, location: SourceLocation) {
        self.entries.push((offset, location));
    }

    /// Sorts entries by offset. Must be called once before [`Self::lookup`].
    pub fn sort(&mut self) {
        self.entries.sort_by_key(|(offset, _)| *offset);
    }

    /// Returns the source location that covers `offset`, i.e. the closest
    /// entry with an offset less than or equal to `offset`.
    #[must_use]
    pub fn lookup(&self, offset: u32) -> Option<&SourceLocation> {
        let idx = self.entries.partition_point(|(addr, _)| *addr <= offset);
        idx.checked_sub(1).map(|i| &self.entries[i].1)
    }

    /// Number of mapped instructions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` when nothing is mapped.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_returns_the_enclosing_instruction() {
        let mut map = SourceMap::new();
        map.insert(0x0000, SourceLocation::new("a.asm", 10, 1));
        map.insert(0x0003, SourceLocation::new("a.asm", 11, 1));
        map.insert(0x0007, SourceLocation::new("a.asm", 12, 1));
        map.sort();

        assert_eq!(map.lookup(0x0000).map(|l| l.line), Some(10));
        assert_eq!(map.lookup(0x0002).map(|l| l.line), Some(10));
        assert_eq!(map.lookup(0x0005).map(|l| l.line), Some(11));
        assert_eq!(map.lookup(0x00FF).map(|l| l.line), Some(12));
    }

    #[test]
    fn lookup_below_first_entry_returns_none() {
        let mut map = SourceMap::new();
        map.insert(0x0010, SourceLocation::new("a.asm", 1, 1));
        map.sort();
        assert!(map.lookup(0x0000).is_none());
    }
}
