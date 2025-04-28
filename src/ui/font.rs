use std::collections::HashMap;

use fontdue::layout::{CoordinateSystem, GlyphRasterConfig, Layout, LayoutSettings, TextStyle};
use rect_packer::{Packer, Rect};

use crate::{asset_loader::FontHandle, graphics::texture::{Texture, TextureFormat}, math::Vector2, misc::{Color32, Rectangle}};

use super::painter::UiPainter;

pub struct FontPainter {
    font: FontHandle,
    atlas: Texture,
    atlas_packer: Packer,
    glyph_cache: HashMap<GlyphRasterConfig, Rectangle>,
    layout: Layout,
}

impl FontPainter {
    pub fn new(font: &FontHandle) -> FontPainter {
        FontPainter {
            font: font.clone(),
            atlas: Texture::new(TextureFormat::RGBA8888, 1024, 1024, 1),
            atlas_packer: Packer::new(rect_packer::Config { width: 1024, height: 1024, border_padding: 0, rectangle_padding: 0 }),
            glyph_cache: HashMap::new(),
            layout: Layout::new(CoordinateSystem::PositiveYDown),
        }
    }

    fn pack_glyph(
        glyph_cache: &mut HashMap<GlyphRasterConfig, Rectangle>,
        font_data: &fontdue::Font,
        atlas_packer: &mut Packer,
        atlas: &mut Texture,
        glyph: GlyphRasterConfig
    )
    {
        if glyph_cache.contains_key(&glyph) {
            return;
        }

        // rasterize glyph & pack into atlas
        let (metrics, bitmap) = font_data.rasterize_config(glyph);
        let rect = if metrics.width == 0 && metrics.height == 0 {
            // note: rect packer fails on zero-sized rectangles
            Rect::new(0, 0, 0, 0)
        }
        else {
            atlas_packer.pack(metrics.width as i32, metrics.height as i32, false).unwrap()
        };
        let rect = Rectangle::new(rect.x, rect.y, rect.width, rect.height);

        let bitmap = bitmap.iter().map(|x| {
            Color32::new(255, 255, 255, *x)
        }).collect::<Vec<_>>();

        atlas.set_texture_data_region(0, rect.x, rect.y, rect.w, rect.h, &bitmap);
        glyph_cache.insert(glyph, rect);
    }

    pub fn draw_string(self: &mut Self, painter: &mut UiPainter,
        text: &str,
        size: f32,
        position: Vector2,
        pivot: Vector2,
        rotation: f32,
        tint: Color32,
        layout: LayoutSettings)
    {
        let style = TextStyle::new(text, size, 0);
        
        self.layout.reset(&layout);
        self.layout.append(&[&self.font.inner], &style);

        // pack glyph rects
        for glyph in self.layout.glyphs() {
            if glyph.char_data.is_whitespace() || glyph.char_data.is_control() {
                continue;
            }

            Self::pack_glyph(
                &mut self.glyph_cache,
                &self.font,
                &mut self.atlas_packer,
                &mut self.atlas,
                glyph.key
            );
        }

        // draw glyphs
        for glyph in self.layout.glyphs() {
            if glyph.char_data.is_whitespace() || glyph.char_data.is_control() {
                continue;
            }

            let glyph_rect = self.glyph_cache[&glyph.key];

            painter.draw_sprite(&self.atlas,
                position,
                Vector2::new(glyph_rect.w as f32, glyph_rect.h as f32),
                pivot - Vector2::new(glyph.x, glyph.y),
                rotation,
                glyph_rect,
                tint);
        }
    }
}