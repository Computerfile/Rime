use std::collections::HashMap;

use crate::{UserOptions, ttf::{font::{FontUserOptions, Glyph}, parser::TTFParser}, window::rasterizer::Rasterizer};

#[derive(Default, Clone, Debug)]
pub struct RasterizedGlyph {
    pub data: Vec<i16>,
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


}
