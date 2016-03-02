//! Text layouting for rendering.
extern crate sdl2;
extern crate sdl2_ttf;

use cursor::Cursor;
use std::rc::Rc;
use sdl2::rect::Rect;
use sdl2_ttf::Font;
use common::WidthOfExt;

/// Find out at which x coordinate to render a cursor in the given line of text.
pub fn cursor_x_pos(col: usize, line: &str, font: Rc<Font>) -> i32 {
    if col >= line.len() {
        font.size_of(line).unwrap().0 as i32
    } else {
        let boundary: Vec<_> = line.char_indices().skip(col-1).take(2).collect();
        let (left_index, left_char) = boundary[0];
        let (right_index, right_char) = boundary[1];
        let left_char_width = font.size_of_char(left_char).unwrap().0;
        let right_char_width = font.size_of_char(right_char).unwrap().0;

        let mut s = String::new();
        s.push(left_char);
        s.push(right_char);
        let combined_width = font.size_of(&s).unwrap().0;

        let single_width = left_char_width + right_char_width;
        let char_offset = if combined_width < single_width {
            0
        } else {
            (combined_width - single_width) / 2
        };
        let line_width = font.size_of(&line[..right_index]).unwrap().0;
        (line_width + char_offset) as i32
    }
}

/// Find out where to render a cursor in the given line of text.
/// Returns a line number and the x position of the cursor in it.
/// The line number is only relevant when the text is being wrapped.
pub fn cursor_pos(col: usize, line: &str, font: Rc<Font>, wrap_width: Option<u32>)
        -> (usize, i32) {
    if col == 0 {
        return (0, 0);
    } else if let Some(ww) = wrap_width {
        let mut col_remainder = col;
        let lines = wrap_line(line, &|t: &str| font.width_of(t), ww);
        let last = lines.len() - 1;
        for (i, line) in lines.into_iter().enumerate() {
            if col_remainder <= line.chars().count() || i == last {
                let x = cursor_x_pos(col_remainder, line, font);
                return (i, x);
            } else {
                col_remainder -= line.chars().count();
            }
        }
        unreachable!();
    } else {
        let x = cursor_x_pos(col, line, font);
        return (0, x);
    }
}

/// Wraps a single (loooong) word to fit within the given line width.
pub fn wrap_word<'a, F>(line: &'a str, width_check: &F, wrap_width: u32)
        -> Vec<&'a str> 
        where F: Fn(&str) -> u32 {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut last_index = 0;
    for (i, ch) in line.char_indices() {
        if width_check(&line[start..i]) > wrap_width {
            let part = &line[start..last_index];
            if part != "" {
                lines.push(part);
            }
            start = last_index;
        } else {
            last_index = i;
        }
    }
    lines.push(&line[start..]);
    lines
}

/// Wrap a line without handling the wrapping of words that are too long to
/// fit on a single line (based on the wrap width).
pub fn wrap_line_simple<'a, F>(text: &'a str, width_check: &F, wrap_width: u32)
        -> Vec<&'a str> 
        where F: Fn(&str) -> u32 {
    if width_check(text) <= wrap_width {
        vec![text]
    } else {
        let mut lines = Vec::new();
        let mut start = 0;
        // The last 'boundary' (whitespace character)
        let mut last_word_start = 0;
        // Ignore whitespace after the first in a sequence
        let mut was_whitespace = true;
        let mut words_read = 0;
        let indices: Vec<_> = text.char_indices().collect();
        for (col, &(i, ch)) in indices.iter().enumerate() {
            if ch.is_whitespace() {
                // Not a whitespace sequence
                if ! was_whitespace {
                    words_read += 1;
                    // Current run too wide for the line
                    if width_check(&text[start..i]) > wrap_width {
                        // Only this word to split
                        if words_read == 1 {
                            if let Some(&(index, _)) = indices[col..]
                                    .iter().find(|&&(i, ch)| {
                                ! ch.is_whitespace()
                            }) {
                                lines.push(&text[start..index]);
                                words_read = 0;
                                start = index;
                                last_word_start = index;
                            } else {
                                break;
                            }
                        } else {
                            let last = &text[start..last_word_start];
                            if last != "" {
                                //println!("More words to split: {}", &text[start..last_word_start]);
                                lines.push(last);
                            }
                    
                            start = last_word_start;
                            words_read = 1;
                        }
                    }
                    was_whitespace = true;
                }
            } else {
                // Set the indices of the first character after whitespace to 'last_word_start'
                if was_whitespace {
                    last_word_start = i;
                }
                was_whitespace = false;
            }
        }
        //println!("Remainder: {}", &text[start..]);
        
        let last = &text[start..last_word_start];
        if last != "" {
            lines.push(last);
            start = last_word_start;
        }
        
        // Push the remainder
        let remainder = &text[start..];
        assert!(remainder != "");
        lines.push(remainder);
        //println!("Lines: {:?}", lines);
        lines
    }
}

/// Splits the given text into lines of text fitting the given wrap width when
/// using the given font.
pub fn wrap_line<'a, F>(text: &'a str, width_check: &F, wrap_width: u32)
        -> Vec<&'a str> 
        where F: Fn(&str) -> u32 {
    let mut lines = wrap_line_simple(text, width_check, wrap_width);
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if width_check(&line) > wrap_width {
            let parts = wrap_word(text, width_check, wrap_width);
            let new_lines = parts.len();
            //println!("'{}' too long => {:?}", line, parts);
            lines[i] = parts[0];
            for (j, part) in parts.into_iter().skip(1).enumerate() {
                lines.insert(i + 1 + j, part);
            }
            i += new_lines;
        } else {
            i += 1;
        }
    }
    lines
}

/// Returns the selection rectangles for the given line and cursors.
pub fn selections(first: Cursor, last: Cursor, lineno: usize, line: &str, 
        font: Rc<Font>, wrap_width: Option<u32>, width: u32) -> Vec<Rect> {
    let mut selections = Vec::new();
    let height = font.recommended_line_height();
    
    // Full selected line
    if first.line < lineno && lineno < last.line {
        let count = if let Some(ww) = wrap_width {
            wrap_line(line, &|t: &str| font.clone().width_of(t), ww).len()
        } else {
            1
        };
        for i in 0 .. count {
            let full_line = Rect::new(
                0, (i as u32 * height) as i32, 
                wrap_width.unwrap_or(width), height
            );
            selections.push(full_line);
        }
    // Selection starts here 
    } else if lineno == first.line {
        // And ends on the same line of text
        if last.line == lineno {
            println!("Selection start and end at {}", lineno);
            let (first_lineno, fx) = cursor_pos(first.col, line, font.clone(), wrap_width);
            let (last_lineno, lx) = cursor_pos(last.col, line, font, wrap_width);
            if last_lineno == first_lineno {
                let rect = Rect::new(
                    fx, (first_lineno as u32 * height) as i32, 
                    (lx - fx) as u32, height
                );
                selections.push(rect);
            } else {
                let first = Rect::new(
                    fx, (first_lineno as u32 * height) as i32,
                    width - fx as u32, height 
                );
                selections.push(first);
                for i in 0 .. (last_lineno - (first_lineno + 1)) {
                    let middle = Rect::new(
                        0, ((first_lineno + 1) as u32 * height) as i32,
                        wrap_width.unwrap(), height
                    );
                    selections.push(middle)
                }
                let last = Rect::new(
                    0, (last_lineno as u32 * height) as i32,
                    lx as u32, height
                );
                selections.push(last);
            }
        } else {
            println!("Selection start at {}", lineno);
            if let Some(ww) = wrap_width {
                let last_line = wrap_line(line, &|t: &str| font.clone().width_of(t), ww).len() - 1;
                let (lineno, x) = cursor_pos(first.col, line, font, wrap_width);
                let start = Rect::new(x, (lineno as u32 * height) as i32, ww - x as u32, height);
                selections.push(start);
                println!("Start: {:?}", start);
                if lineno < last_line {
                    let rest = Rect::new(
                        0, start.bottom(), 
                        ww, (last_line - lineno) as u32 * height
                    );
                    selections.push(rest);
                    println!("Rest: {:?}", rest);
                }
            } else {
                let (_, x) = cursor_pos(first.col, line, font, wrap_width);
                selections.push(Rect::new(x, 0, width - x as u32, height));
            }
        }
    // Selection ends here
    } else if lineno == last.line {
        println!("Selection end at {}", lineno);
        let (lineno, x) = cursor_pos(last.col, line, font, wrap_width);
        if lineno != 0 {
            let pre = Rect::new(0, 0, wrap_width.unwrap(), lineno as u32 * height);
            selections.push(pre);
            println!("Pre: {:?}", pre);
        }
        let last = Rect::new(0, 0, x as u32, height);
        selections.push(last);
        println!("Last: {:?}", last);
    }
    selections
}

