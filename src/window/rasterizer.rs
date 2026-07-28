use crate::{UserOptions, ttf::font::Glyph, window::text_engine::RasterizedGlyph};


pub struct Rasterizer {
    glyph: Glyph,
    user_options: UserOptions,
} 


impl Rasterizer {
    pub fn new(glyph: Glyph, user_options: UserOptions) -> Self {
        Self {
            glyph,
            user_options
        }
    }


    pub fn bitmap_rasterizer(&mut self) -> RasterizedGlyph {
        let width = self.glyph.bounds.x_max - self.glyph.bounds.x_min;
        let height = self.glyph.bounds.y_max - self.glyph.bounds.y_min;

        for i in 0..width {

            for j in 0..height {

            } 

        } 

        RasterizedGlyph::default()
    }

}
