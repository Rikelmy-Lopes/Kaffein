//! A `TextEdit` buffer wrapper that reports mutations to the highlight cache.
//!
//! egui lays text out before and after processing input. Without a mutation
//! counter the layouter cannot tell whether the text it receives is the same
//! text it tokenized last time. The counter makes unchanged frames cheap: the
//! cache is consulted instead of re-tokenizing the file.

use std::cell::Cell;
use std::ops::Range;
use std::rc::Rc;

use egui::TextBuffer;

pub(crate) struct TrackedBuffer<'a> {
    text: &'a mut String,
    generation: Rc<Cell<u64>>,
}

impl<'a> TrackedBuffer<'a> {
    pub(crate) fn new(text: &'a mut String, generation: Rc<Cell<u64>>) -> Self {
        Self { text, generation }
    }

    fn touch(&self) {
        self.generation.set(self.generation.get().wrapping_add(1));
    }
}

impl TextBuffer for TrackedBuffer<'_> {
    fn is_mutable(&self) -> bool {
        true
    }

    fn as_str(&self) -> &str {
        self.text
    }

    fn insert_text(&mut self, text: &str, char_index: usize) -> usize {
        let byte_index = self.byte_index_from_char_index(char_index);
        self.text.insert_str(byte_index, text);
        self.touch();
        text.chars().count()
    }

    fn delete_char_range(&mut self, char_range: Range<usize>) {
        assert!(char_range.start <= char_range.end);
        let start = self.byte_index_from_char_index(char_range.start);
        let end = self.byte_index_from_char_index(char_range.end);
        self.text.replace_range(start..end, "");
        self.touch();
    }

    fn clear(&mut self) {
        self.text.clear();
        self.touch();
    }

    fn replace_with(&mut self, text: &str) {
        self.text.clear();
        self.text.push_str(text);
        self.touch();
    }

    fn take(&mut self) -> String {
        self.touch();
        std::mem::take(self.text)
    }
}
