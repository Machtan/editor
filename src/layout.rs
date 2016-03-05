//! Text layouting for rendering.
extern crate sdl2;
extern crate sdl2_ttf;

use sdl2::rect::Rect;

/// Find out at which x coordinate to render a cursor in the given line of text.
pub fn cursor_x_pos<F>(col: usize, line: &str, width_check: &F)
        -> i32 
        where F: Fn(&str) -> u32 {
    if col == 0 {
        0
    } else if col >= line.len() {
        width_check(line) as i32
    } else {
        let boundary: Vec<_> = line.char_indices().skip(col-1).take(2).collect();
        let (_, left_char) = boundary[0];
        let (right_index, right_char) = boundary[1];
        let mut left_char_string = String::new();
        left_char_string.push(left_char);
        let left_char_width = width_check(&left_char_string);
        let mut right_char_string = String::new();
        right_char_string.push(right_char);
        let right_char_width = width_check(&right_char_string);

        let mut s = String::new();
        s.push(left_char);
        s.push(right_char);
        let combined_width = width_check(&s);

        let single_width = left_char_width + right_char_width;
        let char_offset = if combined_width < single_width {
            0
        } else {
            (combined_width - single_width) / 2
        };
        let line_width = width_check(&line[..right_index]);
        (line_width + char_offset) as i32
    }
}

/// Find out where to render a cursor in the given line of text.
/// Returns a line number and the x position of the cursor in it.
/// The line number is only relevant when the text is being wrapped.
pub fn cursor_pos<F>(col: usize, lines: &Vec<&str>, width_check: &F)
        -> (usize, i32) 
        where F: Fn(&str) -> u32 {
    if col == 0 {
        return (0, 0);
    } else if lines.len() > 1 {
        let mut col_remainder = col;
        let last = lines.len() - 1;
        for (i, line) in lines.iter().enumerate() {
            if col_remainder < line.chars().count() || i == last {
                let x = cursor_x_pos(col_remainder, line, width_check);
                return (i, x);
            } else {
                col_remainder -= line.chars().count();
            }
        }
        unreachable!();
    } else {
        let x = cursor_x_pos(col, lines[0], width_check);
        return (0, x);
    }
}

/// Wraps a single (loooong) word to fit within the given line width.
pub fn wrap_word<'a, F>(line: &'a str, should_wrap: &F)
        -> Vec<&'a str> 
        where F: Fn(&str) -> bool {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut last_index = 0;
    let mut indices: Vec<_> = line.char_indices().skip(1).map(|(i, _)| i).collect();
    indices.push(line.len());
    for next_index in indices {
        if should_wrap(&line[start..next_index]) {
            let part = &line[start..last_index];
            if part != "" {
                //println!("--- '{}'", part);
                lines.push(part);
            }
            start = last_index;
            
        }
        last_index = next_index;
    }
    let part = &line[start..];
    //println!("--- '{}'", part);
    lines.push(part);
    lines
}

/// Splits the given text into lines of text fitting the given wrap width when
/// using the given width checking function.
pub fn wrap_line<'a, F>(line: &'a str, should_wrap: &F)
        -> Vec<&'a str> 
        where F: Fn(&str) -> bool {
    if ! should_wrap(line) {
        vec![line]
    } else {
        let mut lines = Vec::new();
        
        let mut start = 0;
        let mut cur_word_begin = 0;
        let mut cur_word_end = 0;
        let mut last_word_begin = 0;
        
        let mut was_whitespace = false;
        
        // Create indices of each character and each next index. 
        // (where the character ends, instead of where it begins)
        let indices = {
            let mut result = Vec::new();
            let mut indices: Vec<_> = line.char_indices().skip(1).map(|(i, _)| i).collect();
            indices.push(line.len());
            for (i, ch) in line.chars().enumerate() {
                result.push((indices[i], ch));
            }
            result
        };
        
        for &(next_index, ch) in indices.iter() {
            if ch.is_whitespace() {
                if ! was_whitespace { // New spacing begins
                    last_word_begin = cur_word_begin;
                }
                cur_word_begin = next_index;
                was_whitespace = true;
            
            } else {
                if was_whitespace { // New word begins
                    
                    if should_wrap(&line[start..cur_word_end]) {
                        if start == last_word_begin { // Single word read
                            let part = &line[start..cur_word_begin];
                            //println!("- '{}'", part);
                            lines.append(&mut wrap_word(part, should_wrap));
                            start = cur_word_begin;
                        } else { // Two or more words read
                            let part = &line[start..last_word_begin];
                            //println!("- '{}'", part);
                            lines.push(part);
                            start = last_word_begin;
                        }
                    }
                }
                cur_word_end = next_index;
                was_whitespace = false;
            }
        }
        
        //println!("Remainder:");
        // Get the remainder
        // If the line doesn't end with a space character, act as if a new word just started
        if cur_word_begin != line.len() {
            last_word_begin = cur_word_begin;
            cur_word_begin = line.len();
        }
        
        if should_wrap(&line[start..cur_word_end]) {
            if start == last_word_begin {
                let part = &line[start..cur_word_begin];
                //println!("- '{}'", part);
                lines.append(&mut wrap_word(part, should_wrap));
            } else {
                let part = &line[start..last_word_begin];
                //println!("- '{}'", part);
                lines.push(part);
                start = last_word_begin;
                
                if start != line.len() {
                    let part = &line[start..cur_word_begin];
                    //println!("- '{}'", part);
                    lines.push(part);
                }
            }
        } else {
            let part = &line[start..];
            //println!("- '{}'", part);
            lines.push(part);
        }
        lines
    }
}

/// Returns the rectangles of a selection which starts and ends on the same line.
pub fn selection_single_line<F>(lines: &Vec<&str>, start: usize, end: usize, 
        width_check: &F, line_width: u32, line_height: u32) 
        -> Vec<Rect> 
        where F: Fn(&str) -> u32 {
    if lines.len() > 1 {
        let mut selections = Vec::new();
        let (sl, sx) = cursor_pos(start, &lines, width_check);
        let (el, ex) = cursor_pos(end, &lines, width_check);
        if sl == el {
            let rect = Rect::new(
                sx, (sl as u32 * line_height) as i32, 
                (ex - sx) as u32, line_height
            );
            selections.push(rect);
        } else {
            let first = Rect::new(
                sx, (sl as u32 * line_height) as i32, 
                line_width - sx as u32, line_height
            );
            selections.push(first);
            let middle_lines = el - sl - 1;
            if middle_lines != 0 {
                let middle = Rect::new(
                    0, line_height as i32, 
                    line_width, middle_lines as u32 * line_height
                );
                selections.push(middle);
            }
            let last = Rect::new(
                0, ((1 + middle_lines) as u32 * line_height) as i32, 
                ex as u32, line_height
            );
            selections.push(last);
        }
        selections
    } else {
        let sx = cursor_x_pos(start, lines[0], width_check);
        let ex = cursor_x_pos(end, lines[0], width_check);
        vec![Rect::new(sx, 0, (ex - sx) as u32, line_height)]
    }
}

/// Returns the rectangles of the first line of a selection that spans multiple
/// lines.
pub fn selection_first_line<F>(lines: &Vec<&str>, start: usize, width_check: &F, 
        line_width: u32, line_height: u32) 
        -> Vec<Rect> 
        where F: Fn(&str) -> u32 {
    if lines.len() > 1 {
        let mut selections = Vec::new();
        let (lineno, cx) = cursor_pos(start, &lines, width_check);
        let first = Rect::new(
            cx, (lineno as u32 * line_height) as i32,
            line_width - cx as u32, line_height
        );
        selections.push(first);
        
        let remaining_lines = (lines.len() - 1) - lineno;
        if remaining_lines != 0 {
            let remaining = Rect::new(
                0, ((lineno as u32 + 1) * line_height) as i32,
                line_width, remaining_lines as u32 * line_height
            );
            selections.push(remaining);
        }
        selections
    } else {
        let cx = cursor_x_pos(start, lines[0], width_check);
        if cx as u32 >= line_width {
            Vec::new()
        } else {
            vec![Rect::new(cx, 0, line_width - cx as u32, line_height)]
        }
    }
}
/// Returns the rectangles of a line in the middle of a selection that spans 
/// multiple lines.
pub fn selection_middle_line(line_count: usize, line_width: u32, line_height: u32) 
        -> Rect {
    Rect::new(0, 0, line_width, (line_count as u32) * line_height)
}

/// Returns the rectangles of the last line of a selection that spans multiple
/// lines.
pub fn selection_last_line<F>(lines: &Vec<&str>, end: usize, width_check: &F,
        line_width: u32, line_height: u32) 
        -> Vec<Rect> 
        where F: Fn(&str) -> u32 {
    if lines.len() > 1 {
        let mut selections = Vec::new();
        let (lineno, cx) = cursor_pos(end, &lines, width_check);
        let last = Rect::new(
            0, (lineno as u32 * line_height) as i32,
            cx as u32, line_height
        );
        selections.push(last);
        
        if lineno > 0 {
            let remainder = Rect::new(
                0, 0,
                line_width, lineno as u32 * line_height
            );
            selections.push(remainder);
        }
        selections
    } else {
        let cx = cursor_x_pos(end, lines[0], width_check);
        let rect = Rect::new(0, 0, cx as u32, line_height);
        vec![rect]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    const TEXT: &'static str = "\
    123\
    456\
    789\
    0";
    
    const TEXT2: &'static str = "\
    333 22     \
    1 4444  \
    22 333   \
    ";
    
    const TEXT3: &'static str = "\
    123\
    456\
    789\
    0 333 ";
    
    const TEXT4: &'static str = "\
    333 22 \
    1 4444 \
    22 333\
    ";
    
    fn width_check(t: &str) -> u32 {
        t.chars().count() as u32
    }
    
    #[test]
    fn test_wrap_word() {
        let res = wrap_word(TEXT, &width_check, 3);
        assert_eq!(res, vec!["123", "456", "789", "0"]);
    }
    
    #[test]
    fn test_wrap_line_long() {
        let res = wrap_line(TEXT, &width_check, 3);
        assert_eq!(res, vec!["123", "456", "789", "0"]);
    }
    
    #[test]
    fn test_wrap_line_long_multiple() {
        let res = wrap_line(TEXT3, &width_check, 3);
        assert_eq!(res, vec!["123", "456", "789", "0 ", "333 "]);
    }
    
    #[test]
    fn test_wrap_line_long_no_remainder() {
        let res = wrap_line(TEXT, &width_check, 2);
        assert_eq!(res, vec!["12", "34", "56", "78", "90"]);
    }
    
    #[test]
    fn test_wrap_line_multiple_three() {
        let res = wrap_line(TEXT2, &width_check, 3);
        assert_eq!(res, vec!["333 ", "22     ", "1 ", "4444  ", "22 ", "333   "]);
    }
    
    #[test]
    fn test_wrap_line_multiple_six() {
        let res = wrap_line(TEXT2, &width_check, 6);
        assert_eq!(res, vec!["333 22     ", "1 4444  ", "22 333   "]);
    }
    
    #[test]
    fn test_wrap_line_no_trailing_space() {
        let res = wrap_line(TEXT4, &width_check, 6);
        assert_eq!(res, vec!["333 22 ", "1 4444 ", "22 333"]);
    }
    
    #[test]
    fn test_cursor_x_pos_zero() {
        let res = cursor_x_pos(0, "hello", &width_check);
        assert_eq!(res, 0);
    }
    
    #[test]
    fn test_cursor_x_pos_non_zero() {
        let res = cursor_x_pos(2, "hello", &width_check);
        assert_eq!(res, 2);
    }
    
    #[test]
    fn test_cursor_x_pos_past_end() {
        let res = cursor_x_pos(8, "hello", &width_check);
        assert_eq!(res, 5);
    }
    
    #[test]
    fn test_cursor_pos_wrapped_first_line() {
        let lines = wrap_line(TEXT, &width_check, 3);
        let res = cursor_pos(2, &lines, &width_check);
        assert_eq!(res, (0, 2))
    }
    
    #[test]
    fn test_cursor_pos_wrapped_middle_line() {
        let lines = wrap_line(TEXT, &width_check, 3);
        let res = cursor_pos(3, &lines, &width_check);
        assert_eq!(res, (1, 0))
    }
    
    #[test]
    fn test_cursor_pos_wrapped_last_line() {
        let lines = wrap_line(TEXT, &width_check, 3);
        let res = cursor_pos(9, &lines, &width_check);
        assert_eq!(res, (3, 0))
    }
    
    #[test]
    fn test_cursor_pos_wrapped_past_end() {
        let lines = wrap_line(TEXT, &width_check, 3);
        let res = cursor_pos(11, &lines, &width_check);
        assert_eq!(res, (3, 1))
    }
}

