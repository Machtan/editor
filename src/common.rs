
/// A trait to more easily slice strings at character boundaries.
pub trait StringSliceExt {
    fn slice_until(&self, char_index: usize) -> &str;
    fn slice_after(&self, char_index: usize) -> &str;
}

impl StringSliceExt for String {
    /// [..x] for characters.
    fn slice_until(&self, char_index: usize) -> &str {
        let (i, _) = self.char_indices().nth(char_index)
            .unwrap_or((self.len(), 'a'));
        &self[..i]
    }
    
    /// [x..] for characters.
    fn slice_after(&self, char_index: usize) -> &str {
        let (i, _) = self.char_indices().nth(char_index)
            .unwrap_or((self.len(), 'a'));
        &self[i..]
    }
}