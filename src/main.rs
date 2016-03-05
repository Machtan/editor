#![allow(unused)]
extern crate sdl2;
extern crate sdl2_ttf;

mod common;
mod cursor;
mod textfield;
mod layout;
mod render_textfield;

use cursor::Cursor;
use textfield::Textfield;

fn main() {
    let text = "\
    Hello world!\n\
    What's up?\n\
    This is a lot of words that should demonstrate word wrapping adequately\n\
    Thisisaverylongsinglewordtoshowthatthisisalsowrappedcorrectly\n\
    This is a line!";
    
    let mut field = Textfield::new(text);
    
    render_textfield::main(&mut field);
}
