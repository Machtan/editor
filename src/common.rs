//! Common functions, structs and extensions
extern crate sdl2;
extern crate sdl2_ttf;

use std::rc::Rc;
use sdl2::pixels::Color;
use sdl2_ttf::Font;


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

pub trait WidthOfExt {
    fn width_of(&self, text: &str) -> u32;
}
impl WidthOfExt for Font {
    fn width_of(&self, text: &str) -> u32 {
        let (width, _) = self.size_of(text).expect("Could not get font size");
        width
    }
}