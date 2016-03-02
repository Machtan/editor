extern crate sdl2;
extern crate sdl2_ttf;
extern crate glorious;

use std::rc::Rc;
use std::path::Path;
use sdl2::event::Event;
use sdl2::keyboard::{LSHIFTMOD, RSHIFTMOD, LGUIMOD, RGUIMOD};
use sdl2::keyboard::Keycode;
use sdl2::render::Renderer;
use sdl2::rect::{Rect, Point};
use sdl2::pixels::Color;
use sdl2_ttf::Font;
use sdl2::surface::Surface;

use cursor::Cursor;
use textfield::Textfield;

trait WidthOfExt {
    fn width_of(&self, text: &str) -> u32;
}
impl WidthOfExt for Font {
    fn width_of(&self, text: &str) -> u32 {
        let (width, _) = self.size_of(text).expect("Could not get font size");
        width
    }
}

#[derive(Clone)]
pub struct TextStyle {
    pub font: Rc<Font>,
    pub color: Color,
    pub background: Option<Color>,
}

#[derive(Clone)]
pub struct TextfieldStyle {
    pub text: TextStyle,
    pub x_pad: u32,
    pub y_pad: u32,
    pub cursor_color: Color,
    pub selection_color: Color,
    pub background: Option<Color>,
}

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
        let char_offset = (combined_width - single_width) / 2;
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
fn wrap_word<'a, F>(line: &'a str, width_check: &F, wrap_width: u32)
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
fn wrap_line_simple<'a, F>(text: &'a str, width_check: &F, wrap_width: u32)
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
fn wrap_line<'a, F>(text: &'a str, width_check: &F, wrap_width: u32)
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

/// Creates the surfaces (cpu images) of the given text rendered using the 
/// given font and optionally wrapped to a given width limit in pixels.
pub fn line_surfaces<'a>(text: &str, style: &TextStyle, wrap_width: Option<u32>)
        -> Vec<Surface<'a>> {
    let mut surfaces = Vec::new();
    
    if let Some(width) = wrap_width {
        for part in wrap_line(text, &|t: &str| style.font.width_of(t), width) {
            let surface = if let Some(background) = style.background {
                style.font.render(part).shaded(style.color, background).unwrap()
            } else {
                style.font.render(part).blended(style.color).unwrap()
            };
            surfaces.push(surface);
        }
    } else {
        let surface = if let Some(background) = style.background {
            style.font.render(text).shaded(style.color, background).unwrap()
        } else {
            style.font.render(text).blended(style.color).unwrap()
        };
        surfaces.push(surface);
    };
    
    surfaces
}

pub fn selection_full(line: &str, font: Rc<Font>, width: u32, 
        wrap_width: Option<u32>)
        -> Vec<Rect> {
    let mut selections = Vec::new();
    let line_height = font.recommended_line_height();
    if let Some(ww) = wrap_width {
        for i in 0 .. wrap_line(line, &|t: &str| font.width_of(t), ww).len() {
            selections.push(Rect::new(0, i as i32 * line_height as i32, width, line_height));
        }
    } else {
        selections.push(Rect::new(0, line_height as i32, width, line_height))
    }
    selections
}

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
            selections.push(Rect::new(0, (i as u32 * height) as i32, width, height));
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
                        width, height
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

pub fn render_cursor(text: &str, font: Rc<Font>, color: Color, 
        wrap_width: Option<u32>) {
    
}

pub fn render_textfield(field: &Textfield, rect: Rect,
        style: &TextfieldStyle, renderer: &mut Renderer, 
        wrap_width: Option<u32>) {
    
    renderer.set_clip_rect(Some(rect));
    let font = style.text.font.clone();
    
    if let Some(color) = style.background {
        renderer.set_draw_color(color);
        renderer.clear();
    }
    
    let (first, last) = field.cursor.order(&field.selection_marker);
    let has_selection = field.has_selection();
    
    let x = rect.x() + style.x_pad as i32;
    let y = rect.y() + style.y_pad as i32;
    let height = style.text.font.recommended_line_height();
    let mut lineno = 0;
    for line in field.lines.iter() {
        // Selection
        if has_selection {
            if lineno >= first.line && lineno <= last.line {
                renderer.set_draw_color(style.selection_color);
                //println!("Selections:");
                for mut rect in selections(first, last, lineno, line, font.clone(), wrap_width, rect.width()) {
                    //println!("- {:?}", rect);
                    rect.offset(x, y + (lineno as u32 * height) as i32);
                    renderer.fill_rect(rect).expect("Could not draw selection");
                }
            }
        } else if lineno == first.line {
            let (clineno, cx) = cursor_pos(first.col, line, font.clone(), wrap_width);
            let start = Point::new(x + cx, y + ((lineno + clineno) as u32 * height) as i32);
            let end = Point::new(x + cx, y + ((lineno + clineno + 1) as u32 * height) as i32);
            renderer.set_draw_color(style.cursor_color);
            renderer.draw_line(start, end).expect("Could not draw cursor");
        }
        
        
        // Text
        if line.is_empty() {
            lineno += 1;
            continue;
        }
        for surface in line_surfaces(line, &style.text, wrap_width) {
            let target = Rect::new(
                x, y + (lineno as u32 * height) as i32, surface.width(), surface.height()
            );
            let mut texture = renderer.create_texture_from_surface(&surface)
                .expect("Could not create text texture");
            renderer.copy(&mut texture, None, Some(target));
            lineno += 1;
        }
    }
    
    renderer.set_clip_rect(None);
}

pub fn old_render_textfield(field: &Textfield, rect: Rect,
        style: &TextfieldStyle, renderer: &mut Renderer) {
    
    renderer.set_clip_rect(Some(rect));
    renderer.set_draw_color(Color::RGBA(220, 220, 255, 255));
    renderer.clear();
    
    let line_height = style.text.font.recommended_line_height();
    
    // Find the selection
    let (first, last) = field.cursor.order(&field.selection_marker);
        
    let x = rect.x() + style.x_pad as i32;
    let font_height = style.text.font.height() as i32;
    
    // Prepare selections
    renderer.set_draw_color(style.selection_color);
    for (lineno, line) in field.lines.iter().enumerate() {
        let y_pos = rect.y() + style.y_pad as i32 + (lineno as i32 * line_height as i32);
        if field.has_selection() {
            // First line
            if lineno == first.line {
                // Single-line selection
                if last.line == first.line { 
                    let x_left = x + cursor_x_pos(
                        first.col, line, style.text.font.clone()
                    ) as i32;
                    let x_right = x + cursor_x_pos(
                        last.col, line, style.text.font.clone()
                    ) as i32;
                    let width = (x_right - x_left) as u32;
                    let rect = Rect::new(
                        x_left, y_pos, width, line_height as u32
                    );
                    renderer.fill_rect(rect);

                // Multi-line selection
                } else { 
                    let offset = cursor_x_pos(
                        first.col, line, style.text.font.clone()
                    );
                    let rect = Rect::new(
                        x + offset as i32, y_pos, rect.width() - offset as u32,
                        line_height as u32
                    );
                    renderer.fill_rect(rect);
                }
            // Intermediate line
            } else if (first.line < lineno) && (lineno < last.line) {
                let rect = Rect::new(x, y_pos, rect.width(),
                    line_height as u32);
                renderer.fill_rect(rect);
        
            // Final line
            } else if lineno == last.line {
                let offset = cursor_x_pos(
                    last.col, line, style.text.font.clone()
                );
                if offset != 0 {
                    let rect = Rect::new(
                        x, y_pos, offset as u32, line_height as u32
                    );
                    renderer.fill_rect(rect);
                }
            }
        
        // Normal cursor
        } else if lineno == first.line {
            let x_pos = x + cursor_x_pos(
                field.cons_cursor().col, line, style.text.font.clone()
            ) as i32;
            renderer.set_draw_color(style.cursor_color);
            renderer.draw_line(
                Point::new(x_pos, y_pos),
                Point::new(x_pos, y_pos + line_height as i32)
            );
        }
    }
    
    // Prepare lines
    let mut y_pos = rect.y() + style.y_pad as i32;
    for (lineno, line) in field.lines.iter().enumerate() {
        if line.is_empty() {
            y_pos += line_height as i32;
            continue;
        }
        renderer.set_draw_color(style.text.color);
        let surface = style.text.font.render(line).blended(style.text.color).unwrap();
        let mut texture = renderer.create_texture_from_surface(&surface)
            .unwrap();
        let (width, height) = style.text.font.size_of(line).unwrap();
        let target = Rect::new(x, y_pos, width, height);
        
        renderer.copy(&texture, None, Some(target));
        y_pos += line_height as i32;        
    }
    
    renderer.set_clip_rect(None);
}

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;
pub fn main(field: &mut Textfield) {
    let context = sdl2::init().unwrap();
    let video = context.video().unwrap();
    let mut clipboard = String::new();
    let ttf = sdl2_ttf::init().unwrap();
    
    let window = video.window("Editor", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered().build().unwrap();
        
    let mut renderer = window.renderer().build().unwrap();
    
    //let font = ttf.load_font(Path::new("Monoid-Regular.ttf"), 16).unwrap();
    let font_path = "/Library/Fonts/Verdana.ttf";
    let font = ttf.load_font(Path::new(font_path), 16).unwrap();
    println!("height/ascent/descent: {} | {} | {}", 
        font.height(), font.ascent(), font.descent());
    println!("Line skip: {}", font.recommended_line_height());
    
    let pink = Color::RGBA(255, 180, 220, 255);
    let red = Color::RGBA(255, 0, 0, 255);
    let white = Color::RGBA(255, 255, 255, 255);
    let black = Color::RGBA(0, 0, 0, 255);
    let text = "Hello Rust!";
    let surface = font.render(text).blended(red).unwrap();
    let mut texture = renderer.create_texture_from_surface(&surface).unwrap();
    let (width, height) = font.size_of(text).unwrap();
    
    renderer.set_draw_color(white);
    renderer.clear();
    
    let pad = 64;
    let target = Rect::new(pad, pad, width, height);
    let text_style = TextStyle {
        font: Rc::new(font), color: black, background: None,
    };
    
    let style = TextfieldStyle { 
        text: text_style, x_pad: 10, y_pad: 10,
        cursor_color: red, selection_color: pink, 
        background: Some(Color::RGBA(220, 220, 255, 255)),
    };
    
    renderer.copy(&mut texture, None, Some(target));
    renderer.present();
    let mut limiter = glorious::FrameLimiter::new(30);
    
    'mainloop: loop {
        for event in context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit{..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                    break 'mainloop;
                },
                Event::KeyDown { keycode, keymod, ..} => {
                    if keymod.is_empty() {
                        match keycode {
                            Some(Keycode::Left) => {
                                field.left();
                            },
                            Some(Keycode::Right) => {
                                field.right();
                            },
                            Some(Keycode::Up) => {
                                field.up();
                            },
                            Some(Keycode::Down) => {
                                field.down();
                            },
                            Some(Keycode::Backspace) => {
                                field.delete_previous();
                            },
                            Some(Keycode::Delete) => {
                                field.delete_next();
                            },
                            Some(Keycode::Return) => {
                                field.insert("\n");
                            }
                            other => {
                                println!("Key down: {:?}", other);
                            },
                        }
                    } else if keymod == RSHIFTMOD || keymod == LSHIFTMOD {
                        match keycode {
                            Some(Keycode::Left) => {
                                field.select_left();
                            },
                            Some(Keycode::Right) => {
                                field.select_right();
                            },
                            Some(Keycode::Up) => {
                                field.select_up();
                            },
                            Some(Keycode::Down) => {
                                field.select_down();
                            },
                            _ => {},
                        }
                    } else if keymod == RGUIMOD || keymod == LGUIMOD {
                        match keycode {
                            Some(Keycode::C) => {
                                let selection = field.selected_text();
                                clipboard = String::from(&selection[..]);
                            },
                            Some(Keycode::X) => {
                                let selection = field.selected_text();
                                field.delete_selection();
                                clipboard = String::from(&selection[..]);
                            },
                            Some(Keycode::V) => {
                                field.insert(&clipboard);
                            },
                            Some(Keycode::Return) => {
                                println!("Text:");
                                println!("{}", clipboard);
                            }
                            _ => {},
                        }
                    }
                    
                },
                Event::TextInput { text, ..} => {
                    println!("Inserting text {:?}", &text);
                    field.insert(&text);
                }
                _ => {}
            }
        }
        
        // Render
        renderer.set_draw_color(white);
        renderer.clear();
        let rect = Rect::new(64, 64, SCREEN_WIDTH - 128,
            SCREEN_HEIGHT - 128);
        //old_render_textfield(field, rect, &style, &mut renderer);
        // 4% cpu at 30 FPS when in debug mode
        render_textfield(field, rect, &style, &mut renderer, Some(200));
        renderer.present();
        
        limiter.limit();
    }
}