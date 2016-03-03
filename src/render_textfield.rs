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
use common::WidthOfExt;
use layout::{cursor_x_pos, cursor_pos, wrap_line, selections};

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

/// Renders the given text field inside the given rect wrapping text at the
/// given wrap_width
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
                for mut rect in selections(first, last, lineno, line, 
                    &|t: &str| font.width_of(t), wrap_width, rect.width(), height
                ) {
                    //println!("- {:?}", rect);
                    rect.offset(x, y + (lineno as u32 * height) as i32);
                    renderer.fill_rect(rect).expect("Could not draw selection");
                }
            }
        } else if lineno == first.line {
            let (clineno, cx) = cursor_pos(first.col, line, &|t: &str| font.width_of(t), wrap_width);
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
                        first.col, line, &|t: &str| style.text.font.width_of(t)
                    ) as i32;
                    let x_right = x + cursor_x_pos(
                        last.col, line, &|t: &str| style.text.font.width_of(t)
                    ) as i32;
                    let width = (x_right - x_left) as u32;
                    let rect = Rect::new(
                        x_left, y_pos, width, line_height as u32
                    );
                    renderer.fill_rect(rect);

                // Multi-line selection
                } else { 
                    let offset = cursor_x_pos(
                        first.col, line, &|t: &str| style.text.font.width_of(t)
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
                    last.col, line, &|t: &str| style.text.font.width_of(t)
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
                field.cons_cursor().col, line, &|t: &str| style.text.font.width_of(t)
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