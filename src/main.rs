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
    println!("Hello, world!");
    let text = "\
    Hello world!\n\
    What's up?\n\
    This is a lot of words that should demonstrate word wrapping adequately\n\
    Thisisaverylongsinglewordtoshowthatthisisalsowrappedcorrectly\n\
    This is a line!";
    let source: Vec<String> = text.lines().map(|s| s.to_string()).collect();
    let mut cursor = Cursor::new(1, 5);
    println!("{}", cursor);
    for _ in 0..20 {
        cursor.left(&source);
        //cursor.debug(&source);
    }
    println!("========================");
    for _ in 0..20 {
        cursor.right(&source);
        //cursor.debug(&source);
    }
    let mut copy = cursor.clone();
    copy.right(&source);
    println!("Cursor: {}, Copy: {}", cursor, copy);

    cursor.debug(&source);
    cursor.down(&source);
    cursor.debug(&source);
    cursor.down(&source);
    cursor.debug(&source);
    cursor.up();
    cursor.debug(&source);
    
    let mut field = Textfield::new(text);
    
    render_textfield::main(&mut field);
}
