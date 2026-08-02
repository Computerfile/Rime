use std::collections::HashMap;

use crate::{UserOptions, ttf::{font::{FontUserOptions, Glyph, GlyphBounds}, parser::TTFParser}, window::rasterizer::Rasterizer};

#[derive(Default, Clone, Debug)]
pub struct RasterizedGlyph {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,

    pub bounds: GlyphBounds,
}

pub struct TextEngine {
    parser: TTFParser,
    rasterized_glyph_cache: HashMap<u32, RasterizedGlyph>,
    user_options: UserOptions,
}


impl TextEngine {
    pub fn new(parser: TTFParser, user_options: UserOptions) -> Self {
        Self {
            parser,
            rasterized_glyph_cache: HashMap::new(),
            user_options
        } 

    }

    pub fn get_rasterized(&mut self, codepoint: u32) -> RasterizedGlyph {
        if self.rasterized_glyph_cache.contains_key(&codepoint) {
            let data = self.rasterized_glyph_cache.get(&codepoint).unwrap().clone();
            data
        }else {
            let glyph_data: Glyph = self.parser.fetch_char_from_cache(codepoint).ok().unwrap();

            let mut r: Rasterizer = Rasterizer::new(glyph_data, self.user_options.clone());

            let rasterized_data: RasterizedGlyph = r.bitmap_rasterizer();

            self.rasterized_glyph_cache.insert(codepoint, rasterized_data);

            self.rasterized_glyph_cache.get(&codepoint).unwrap().clone()
        }


    }

    pub fn get_advaned_widths(&mut self, codepoint: u32) -> Option<u16> {
        
            
        if self.parser.font_limits.h_metrics.advanced_widths.contains_key(&codepoint) {
            
            let aw = self.parser.font_limits.h_metrics.advanced_widths.get(&codepoint).unwrap();
            
            Some(*aw)
        }else {
            let _ = self.parser.fetch_char_from_cache(codepoint);
            self.parser.font_limits.h_metrics.advanced_widths.get(&codepoint).copied()
        }

    }


    pub fn get_lsb(&mut self, codepoint: u32) -> Option<i16> {
        if self.parser.font_limits.h_metrics.left_side_bearings.contains_key(&codepoint) {
            let aw = self.parser.font_limits.h_metrics.left_side_bearings.get(&codepoint).unwrap();
            Some(*aw)
        }else {
            let _ = self.parser.fetch_char_from_cache(codepoint);
            self.parser.font_limits.h_metrics.left_side_bearings.get(&codepoint).copied()
        }

    }



}
