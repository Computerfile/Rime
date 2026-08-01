use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct AtlasRect {
    pub x: u32,
    pub y: u32,
    pub width: u32, 
    pub height: u32,
} 

pub struct GlyphCache {
    glyph_cache: HashMap<u32, AtlasRect>,
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
    pub atlas_width: u32,
    pub atlas_height: u32,
}


impl GlyphCache {

    pub fn new(atlas_width: u32, atlas_height: u32) -> Self {
        let glyph_cache: HashMap<u32, AtlasRect> = HashMap::new();
        Self { 
            glyph_cache: glyph_cache,
            cursor_x: 0, 
            cursor_y: 0, 
            row_height: 0,
            atlas_width: atlas_width,
            atlas_height: atlas_height,
        }
    }

    pub fn add_to_cache(&mut self, codepoint: u32, rect: AtlasRect) {
        self.glyph_cache.insert(codepoint, rect);
    }

    pub fn get(&self, codepoint: u32) -> Option<&AtlasRect> {
        self.glyph_cache.get(&codepoint)
    }
    
    pub fn get_size(&self) -> u32 {
        self.atlas_width * self.atlas_height
    }

    pub fn get_or_allocate(&mut self, codepoint: u32, glyph_width: u32, glyph_height: u32) -> Option<AtlasRect> {
        if let Some(r) = self.get(codepoint) {
            return Some(*r);
        }

        if glyph_width > (self.atlas_width - self.cursor_x) {
            self.cursor_x = 0;
            self.cursor_y += self.row_height;
            self.row_height = 0;
        }

        if self.cursor_y + glyph_height > self.atlas_height {
            return None;
        }

        let allocated = AtlasRect {
            x: self.cursor_x,
            y: self.cursor_y,
            width: glyph_width,
            height: glyph_height,
        };
        self.cursor_x += glyph_width;
        self.row_height = self.row_height.max(glyph_height);
        self.add_to_cache(codepoint, allocated);
        Some(allocated)
    }
}
