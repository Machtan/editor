extern crate sdl2;
extern crate sdl2_ttf;
extern crate glorious;

use std::rc::Rc;
use std::path::Path;
use std::collections::HashMap;
use sdl2::event::Event;
use sdl2::keyboard::{LSHIFTMOD, RSHIFTMOD, LGUIMOD, RGUIMOD};
use sdl2::keyboard::Keycode;
use sdl2::render::{Renderer, Texture, TextureQuery};
use sdl2::rect::{Rect, Point};
use sdl2::pixels::Color;
use sdl2_ttf::Font;
use sdl2::surface::Surface;

use textfield::Textfield;
use common::WidthOfExt;
use layout::{cursor_x_pos, cursor_pos, wrap_line};
use layout::{selection_single_line, selection_first_line, selection_middle_line, selection_last_line};

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



/// Creates the surfaces (cpu images) of the given text rendered using the 
/// given font and optionally wrapped to a given width limit in pixels.
pub fn line_surface<'a>(line: &str, style: &TextStyle) -> Surface<'a> {
    if let Some(background) = style.background {
        style.font.render(line).shaded(style.color, background).unwrap()
    } else {
        style.font.render(line).blended(style.color).unwrap()
    }
}

const ASCII_CHARS: [char; 95] = [
    ' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.',
    '/', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=',
    '>', '?', '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[',
    '\\', ']', '^', '_', '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j',
    'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y',
    'z', '{', '|', '}', '~',
];
fn max_ascii_char_width(font: Rc<Font>) -> u32 {
    let mut max = 0;
    for &ch in ASCII_CHARS.iter() {
        let (width, _) = font.size_of_char(ch)
            .expect("Could not size ascii char");
        if width > max {
            max = width;
        }
    }
    max
}

/// Renders the given text field inside the given rect wrapping text at the
/// given wrap_width.
pub fn render_textfield<'a>(field: &Textfield, rect: Rect,
        style: &TextfieldStyle, renderer: &mut Renderer, 
        wrap_width: Option<u32>, max_char_width: Option<u32>,
        text_cache: &mut HashMap<String, Texture>, 
        wrap_cache: &mut HashMap<String, Vec<usize>>) {
    
    renderer.set_clip_rect(Some(rect));
    
    if let Some(color) = style.background {
        renderer.set_draw_color(color);
        renderer.clear();
    }
    
    let (first, last) = field.cursor.order(&field.selection_marker);
    let has_selection = field.has_selection();
    
    let x = rect.x() + style.x_pad as i32;
    let y = rect.y() + style.y_pad as i32;
    let height = style.text.font.recommended_line_height();
    let mut visual_lineno = 0;
    // Use a heuristic to skip the glyph size check
    let should_wrap = |t: &str| {
        if let Some(width) = max_char_width {
            if t.len() as u32 * width <= wrap_width.unwrap() {
                false
            } else {
                style.text.font.width_of(t) > wrap_width.unwrap()
            }
        } else {
            style.text.font.width_of(t) > wrap_width.unwrap()
        }
    };
    let width_check = |t: &str| {
        style.text.font.width_of(t)
    };
    let width = wrap_width.unwrap_or(rect.width() - style.x_pad * 2);
    
    for (lineno, line) in field.lines.iter().enumerate() {
        let y_pos = y + (visual_lineno as u32 * height) as i32;
        let lines = if let Some(wrap_width) = wrap_width {
            let indices = wrap_cache.entry(line.clone())
                .or_insert_with(|| wrap_line(line, &should_wrap));
            if ! indices.is_empty() {
                let mut lines = Vec::new();
                let mut start = 0;
                for &mut index in indices {
                    lines.push(&line[start..index]);
                    start = index;
                }
                lines.push(&line[start..]);
                lines
            } else {
                vec![line.as_str()]
            }
        } else {
            vec![line.as_str()]
        };
        // Selection
        if has_selection {
            // Same line
            if lineno == first.line && lineno == last.line {
                renderer.set_draw_color(style.selection_color);
                for mut sel in selection_single_line(&lines, first.col, last.col,
                        &width_check, width, height) {
                    sel.offset(x, y_pos);
                    renderer.fill_rect(sel).expect("Selection fill rect");
                }
            // First line
            } else if lineno == first.line {
                renderer.set_draw_color(style.selection_color);
                for mut sel in selection_first_line(&lines, first.col, 
                        &width_check, width, height) {
                    sel.offset(x, y_pos);
                    renderer.fill_rect(sel).expect("Selection fill rect");
                }
            // Last line
            } else if lineno == last.line {
                renderer.set_draw_color(style.selection_color);
                for mut sel in selection_last_line(&lines, last.col, 
                        &width_check, width, height) {
                    sel.offset(x, y_pos);
                    renderer.fill_rect(sel).expect("Selection fill rect");
                }
            // Middle line
            } else if first.line < lineno && lineno < last.line {
                renderer.set_draw_color(style.selection_color);
                let mut sel = selection_middle_line(lines.len(), width, height);
                sel.offset(x, y_pos);
                renderer.fill_rect(sel).expect("Selection fill rect");
            }
        
        // Cursor
        } else if lineno == first.line {
            let (cx, cy) = if let Some(wrap_width) = wrap_width {
                let (cl, cx) = cursor_pos(first.col, &lines, &width_check);
                (x + cx, y_pos + (cl as u32 * height) as i32)
            } else {
                let cx = cursor_x_pos(first.col, line, &width_check);
                (x + cx, y_pos)
            };
            
            let start = Point::new(cx, cy);
            let end = Point::new(cx, cy + height as i32);
            renderer.set_draw_color(style.cursor_color);
            renderer.draw_line(start, end).expect("Could not draw cursor");
        }
        
        
        // Text
        let line_count = lines.len();
        for (i, line) in lines.into_iter().enumerate() {
            if line.is_empty() {
                continue;
            }
            let mut texture = text_cache.entry(String::from(line)).or_insert_with(|| {
                let surface = line_surface(line, &style.text);
                renderer.create_texture_from_surface(surface)
                    .expect("Could not create text texture")
            });
            let TextureQuery { width: w, height: h, ..} = texture.query();
            let target = Rect::new(
                x, y_pos + (i as u32 * height) as i32, 
                w, h
            );
            
            renderer.copy(&mut texture, None, Some(target));
        }
        visual_lineno += line_count;
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
    
    let clear_color = white;
    renderer.set_draw_color(clear_color);
    renderer.clear();
    
    let text_style = TextStyle {
        font: Rc::new(font), color: black, background: None,
    };
    
    let style = TextfieldStyle { 
        text: text_style, x_pad: 10, y_pad: 10,
        cursor_color: red, selection_color: pink, 
        background: Some(Color::RGBA(220, 220, 255, 255)),
    };
    
    renderer.present();
    let mut limiter = glorious::FrameLimiter::new(30);
    let mut dirty = true;
    let mut text_cache = HashMap::new();
    let mut wrap_cache = HashMap::new();
    let max_char_width = max_ascii_char_width(style.text.font.clone());
    
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
                                dirty = true;
                            },
                            Some(Keycode::Right) => {
                                field.right();
                                dirty = true;
                            },
                            Some(Keycode::Up) => {
                                field.up();
                                dirty = true;
                            },
                            Some(Keycode::Down) => {
                                field.down();
                                dirty = true;
                            },
                            Some(Keycode::Backspace) => {
                                field.delete_previous();
                                dirty = true;
                            },
                            Some(Keycode::Delete) => {
                                field.delete_next();
                                dirty = true;
                            },
                            Some(Keycode::Return) => {
                                field.insert("\n");
                                dirty = true;
                            }
                            other => {
                                println!("Key down: {:?}", other);
                            },
                        }
                    } else if keymod == RSHIFTMOD || keymod == LSHIFTMOD {
                        match keycode {
                            Some(Keycode::Left) => {
                                field.select_left();
                                dirty = true;
                            },
                            Some(Keycode::Right) => {
                                field.select_right();
                                dirty = true;
                            },
                            Some(Keycode::Up) => {
                                field.select_up();
                                dirty = true;
                            },
                            Some(Keycode::Down) => {
                                field.select_down();
                                dirty = true;
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
                    dirty = true;
                }
                _ => {}
            }
        }
        
        /* Performance stuff
        CPU usage:
            Non-wrapped: 8.5%
            Wrapped: 11.5% 
                No selection render: 11.1
                Full selection: 13
                No text: 2
                No text render: 6
                
        
        Scales linearly with more text:
        When I fill the window with short lines it becomes 18%
        */
        
        // Render
        let rect = Rect::new(64, 64, SCREEN_WIDTH - 128,
            SCREEN_HEIGHT - 128);

        let wrap_width = Some(200);
        // Dirty check not used atm to better improve general performance
        if true { // dirty.
            renderer.set_draw_color(clear_color);
            renderer.clear();
            render_textfield(field, rect, &style, &mut renderer, wrap_width, 
                Some(max_char_width), &mut text_cache, &mut wrap_cache);
            renderer.present();
            dirty = false;
        }
        
        limiter.limit();
    }
}