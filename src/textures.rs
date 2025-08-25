use raylib::prelude::*;
use std::collections::HashMap;
use image::{GenericImageView, DynamicImage, imageops::FilterType};

struct TextureEntry {
    data: Vec<u8>, 
    width: usize,
    height: usize,
    texture: Texture2D,
}

pub struct TextureManager {
    entries: HashMap<char, TextureEntry>,
}

impl TextureManager {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let mut entries = HashMap::new();

        let texture_files = vec![
            ('+', "assets/textura_pared.png"),
            ('-', "assets/textura_pared.png"),
            ('|', "assets/textura_pared.png"),
            ('D', "assets/door.png"),
            ('g', "assets/textura_pared.png"),
            ('#', "assets/textura_pared.png"),
        ];

        for (ch, path) in texture_files {
            let dynimg = match image::open(path) {
                Ok(i) => i,
                Err(e) => panic!("TextureManager: no pude cargar {}: {:?}", path, e),
            };

            let (w0, h0) = dynimg.dimensions();
            let max_dim = 512u32;
            let img_rgba: DynamicImage = if w0 > max_dim || h0 > max_dim {
                let scale = (max_dim as f32) / (w0 as f32).max(h0 as f32);
                let new_w = ((w0 as f32) * scale).max(1.0) as u32;
                let new_h = ((h0 as f32) * scale).max(1.0) as u32;
                let small = image::imageops::resize(&dynimg.to_rgba8(), new_w, new_h, FilterType::Triangle);
                DynamicImage::ImageRgba8(small)
            } else {
                dynimg.to_rgba8().into()
            };

            let rgba = img_rgba.to_rgba8();
            let (w, h) = rgba.dimensions();
            let buf = rgba.into_raw();

            let texture = match rl.load_texture(thread, path) {
                Ok(t) => t,
                Err(_) => {
                    rl.load_texture_from_image(thread, &Image::gen_image_color(1, 1, Color::WHITE))
                        .expect("failed fallback texture")
                }
            };

            entries.insert(ch, TextureEntry {
                data: buf,
                width: w as usize,
                height: h as usize,
                texture,
            });
        }

        TextureManager { entries }
    }

    pub fn sample_char(&self, ch: char, u: f32, v: f32) -> Color {
        if let Some(entry) = self.entries.get(&ch) {
            let uu = (u.fract() + 1.0).fract();
            let vv = (v.fract() + 1.0).fract();

            let tx = ((uu * (entry.width.saturating_sub(1) as f32)).round() as usize).min(entry.width.saturating_sub(1));
            let ty = ((vv * (entry.height.saturating_sub(1) as f32)).round() as usize).min(entry.height.saturating_sub(1));

            let idx = (ty * entry.width + tx).saturating_mul(4);
            if idx + 3 < entry.data.len() {
                let r = entry.data[idx];
                let g = entry.data[idx + 1];
                let b = entry.data[idx + 2];
                let a = entry.data[idx + 3];
                return Color::new(r, g, b, a);
            } else {
                return Color::WHITE;
            }
        }
        Color::WHITE
    }

    pub fn get_texture(&self, ch: char) -> Option<&Texture2D> {
        self.entries.get(&ch).map(|e| &e.texture)
    }

    pub fn tex_size(&self, ch: char) -> Option<(usize,usize)> {
        self.entries.get(&ch).map(|e| (e.width, e.height))
    }
}
