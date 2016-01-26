extern crate sdl2;
extern crate sdl2_ttf;

use std::path::Path;
use sdl2::event::Event;
use sdl2::keyboard::{LSHIFTMOD, RSHIFTMOD};
use sdl2::keyboard::Keycode;
use sdl2::render::Renderer;
use sdl2::rect::{Rect, Point};
use sdl2::pixels::Color;
use sdl2_ttf::Font;

use textfield::Textfield;

pub struct TextfieldStyle<'a> {
    pub font: &'a Font,
    pub color: Color,
    pub x_pad: u32,
    pub y_pad: u32,
    pub selection_color: Color,
}

fn get_cursor_pos(col: usize, text: &str, font: &Font) -> u32 {
    if col == 0 {
        0
    } else if col >= text.len() {
        font.size_of(text).unwrap().0
    } else {
        let boundary: Vec<_> = text.char_indices().skip(col-1).take(2).collect();
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
        let text_width = font.size_of(&text[..right_index]).unwrap().0;
        text_width + char_offset
    }
}

pub fn render(field: &Textfield, x: i32, y: i32, style: &TextfieldStyle, 
        renderer: &mut Renderer) {
    let spacing = style.font.recommended_line_spacing();
    
    // Find the selection
    let (first, last) = if field.cursor < field.selection_marker {
        (field.cursor.clone(), field.selection_marker.clone())
    } else {
        (field.selection_marker.clone(), field.cursor.clone())
    };
    
    let mut textures = Vec::new();
    let mut selections: Vec<Rect> = Vec::new();
    let mut cursor_pos = None;
    
    let x = x + style.x_pad as i32;
    let font_height = style.font.height() as i32;
    let mut max_width = 0;
    let mut y_pos = y + style.y_pad as i32;
    
    // Prepare lines
    for (lineno, line) in field.lines.iter().enumerate() {
        if line.is_empty() {
            continue;
        }
        renderer.set_draw_color(style.color);
        let surface = style.font.render(line).blended(style.color).unwrap();
        let mut texture = renderer.create_texture_from_surface(&surface)
            .unwrap();
        let (width, height) = style.font.size_of(line).unwrap();
        let target = Rect::new_unwrap(x, y_pos, width, height);
        textures.push((texture, target));
        
        if width > max_width {
            max_width = width;
        }
        y_pos += spacing;        
    }
    
    // Prepare selections
    for (lineno, line) in field.lines.iter().enumerate() {
        let y_pos = y + style.y_pad as i32 + (lineno as i32 * spacing);
        if field.has_selection() {
            // First line
            if lineno == first.line {
                // Single-line selection
                if last.line == first.line { 
                    let x_left = x + get_cursor_pos(
                        first.col, line, style.font
                    ) as i32;
                    let x_right = x + get_cursor_pos(
                        last.col, line, style.font
                    ) as i32;
                    let width = (x_right - x_left) as u32;
                    let rect = Rect::new_unwrap(
                        x_left, y_pos, width, spacing as u32
                    );
                    selections.push(rect);

                // Multi-line selection
                } else { 
                    let offset = get_cursor_pos(
                        first.col, line, style.font
                    );
                    let width = max_width - offset;
                    let rect = Rect::new_unwrap(
                        x + offset as i32, y_pos, width, spacing as u32
                    );
                    selections.push(rect);
                }
            // Intermediate line
            } else if (first.line < lineno) && (lineno < last.line) {
                let rect = Rect::new_unwrap(x, y_pos, max_width, spacing as u32);
                selections.push(rect);
        
            // Final line
            } else if lineno == last.line {
                let offset = get_cursor_pos(
                    last.col, line, style.font
                );
                if offset != 0 {
                    let rect = Rect::new_unwrap(
                        x, y_pos, offset, spacing as u32
                    );
                    selections.push(rect);
                }
            }
        
        // Normal cursor
        } else if lineno == first.line {
            let x_pos = x + get_cursor_pos(
                field.cons_cursor().col, line, style.font
            ) as i32;
            cursor_pos = Some((x_pos, y_pos));
        }
    }
    
    renderer.set_draw_color(style.selection_color);
    for selection in selections {
        renderer.draw_rect(selection);
    }
    
    for (texture, target) in textures {
        renderer.copy(&texture, None, Some(target));
    }
    
    if let Some((x_pos, y_pos)) = cursor_pos {
        renderer.draw_line(
            Point::new(x_pos, y_pos),
            Point::new(x_pos, y_pos + spacing)
        );
    }
}

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;
pub fn main(field: &mut Textfield) {
    let context = sdl2::init().unwrap();
    let video = context.video().unwrap();
    let ttf = sdl2_ttf::init().unwrap();
    
    let window = video.window("Editor", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered().build().unwrap();
        
    let mut renderer = window.renderer().build().unwrap();
    
    let font = ttf.load_font(Path::new("Monoid-Regular.ttf"), 16).unwrap();
    println!("height/ascent/descent: {} | {} | {}", 
        font.height(), font.ascent(), font.descent());
    println!("Line skip: {}", font.recommended_line_spacing());
    
    let pink = Color::RGBA(255, 180, 220, 255);
    let red = Color::RGBA(255, 0, 0, 255);
    let white = Color::RGBA(255, 255, 255, 255);
    let text = "Hello Rust!";
    let surface = font.render(text).blended(red).unwrap();
    let mut texture = renderer.create_texture_from_surface(&surface).unwrap();
    let (width, height) = font.size_of(text).unwrap();
    
    renderer.set_draw_color(white);
    renderer.clear();
    
    let pad = 64;
    let target = Rect::new_unwrap(pad, pad, width, height);
    let style = TextfieldStyle { 
        font: &font, color: red, x_pad: 10, y_pad: 10,
        selection_color: pink, 
    };
    
    renderer.copy(&mut texture, None, Some(target));
    renderer.present();
    
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
                                field.delete();
                            },
                            Some(Keycode::Delete) => {
                                field.delete_forward();
                            },
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
                    }
                    
                },
                _ => {}
            }
        }
        
        // Render
        renderer.set_draw_color(white);
        renderer.clear();
        render(field, 64, 64, &style, &mut renderer);
        renderer.present();
    }
}