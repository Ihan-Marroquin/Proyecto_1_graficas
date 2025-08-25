use raylib::prelude::*;
use raylib::{RaylibHandle, RaylibThread};

pub struct Framebuffer {
    width: u32,
    height: u32,
    color_buffer: Image,
    background_color: Color,
    current_color: Color,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32, background_color: Color) -> Self {
        let color_buffer = Image::gen_image_color(width as i32, height as i32, background_color);
        Framebuffer {
            width,
            height,
            color_buffer,
            background_color,
            current_color: Color::WHITE,
        }
    }

    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    pub fn background_color(&self) -> Color { self.background_color }

    pub fn clear(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                Image::draw_pixel(&mut self.color_buffer, x as i32, y as i32, self.background_color);
            }
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32) {
        if x < self.width && y < self.height {
            Image::draw_pixel(&mut self.color_buffer, x as i32, y as i32, self.current_color);
        }
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
        self.clear();
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn swap_buffers_with_fps(
        &self,
        window: &mut RaylibHandle,
        raylib_thread: &RaylibThread,
        fps_text: Option<&str>,
        stamina_opt: Option<(f32, f32)>,
        health_opt: Option<(f32, f32)>,
        shield_opt: Option<(f32, f32)>,
        sprite_draws: Option<&[(&Texture2D, f32, i32, i32)]>,
    ) {
        if let Ok(texture) = window.load_texture_from_image(raylib_thread, &self.color_buffer) {
            let mut d = window.begin_drawing(raylib_thread);
            d.draw_texture(&texture, 0, 0, Color::WHITE);

            if let Some(list) = sprite_draws {
                for &(tex, scale, sx, sy) in list.iter() {
                    let pos = Vector2::new(sx as f32, sy as f32);
                    d.draw_texture_ex(tex, pos, 0.0, scale, Color::WHITE);
                }
            }

            if let Some(fps) = fps_text {
                let margin = 10;
                let font_size = 20;
                let tw = d.measure_text(fps, font_size);
                let tx = (self.width as i32) - tw - margin;
                let ty = margin;
                d.draw_text(fps, tx, ty, font_size, Color::DARKGRAY);
            }

            if let Some((current, max)) = stamina_opt {
                let margin = 10;
                let bar_w = 300;
                let bar_h = 18;
                let x = ((self.width as i32) - bar_w) / 2;
                let y = margin;

                d.draw_rectangle(x, y, bar_w, bar_h, Color::LIGHTGRAY);
                let pct = (current / max).clamp(0.0, 1.0);
                let fill_w = (pct * (bar_w as f32)).round() as i32;
                d.draw_rectangle(x, y, fill_w, bar_h, Color::GREEN);
                d.draw_rectangle_lines(x, y, bar_w, bar_h, Color::DARKGRAY);

                let label = "Estamina";
                let ltw = d.measure_text(label, 16);
                d.draw_text(label, x + (bar_w/2) - (ltw/2), y + bar_h + 2, 16, Color::DARKGRAY);
            }

            if let Some((current, max)) = health_opt {
                let margin = 10;
                let bar_w = 200;
                let bar_h = 18;
                let y = margin + 18 + 8; 
                let x = margin;

                d.draw_rectangle(x, y, bar_w, bar_h, Color::LIGHTGRAY);
                let pct = (current / max).clamp(0.0, 1.0);
                let fill_w = (pct * (bar_w as f32)).round() as i32;
                d.draw_rectangle(x, y, fill_w, bar_h, Color::RED);
                d.draw_rectangle_lines(x, y, bar_w, bar_h, Color::DARKGRAY);

                let label = format!("HP: {}/{}", current.round() as i32, max.round() as i32);
                d.draw_text(&label, x + bar_w + 8, y, 18, Color::DARKGRAY);

                if let Some((scur, smax)) = shield_opt {
                    if scur > 0.0 {
                        let sy = y + bar_h + 6;
                        d.draw_rectangle(x, sy, bar_w, bar_h, Color::LIGHTGRAY);
                        let spct = (scur / smax).clamp(0.0, 1.0);
                        let sfill = (spct * (bar_w as f32)).round() as i32;
                        d.draw_rectangle(x, sy, sfill, bar_h, Color::SKYBLUE);
                        d.draw_rectangle_lines(x, sy, bar_w, bar_h, Color::DARKGRAY);
                        let slabel = format!("Shield: {}/{}", scur.round() as i32, smax.round() as i32);
                        d.draw_text(&slabel, x + bar_w + 8, sy, 16, Color::DARKGRAY);
                    }
                }
            }
        }
    }

    pub fn swap_buffers(
        &self,
        window: &mut RaylibHandle,
        raylib_thread: &RaylibThread,
    ) {
        self.swap_buffers_with_fps(window, raylib_thread, None, None, None, None, None);
    }
}
