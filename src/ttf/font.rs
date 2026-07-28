#[derive(Default, Debug, Clone)]
pub struct FontUserOptions {
   pub path: String,
} 


#[derive(Default, Debug)]
pub struct FontMetric {
    pub units_per_em: u16,
    pub long_loca: bool
}

impl FontMetric {
    

}


#[derive(Default, Debug, Clone)]
pub struct GlyphPoint {
    pub x: i16, 
    pub y: i16,
    pub on_curve: bool,
}

#[derive(Default, Debug, Clone)]
pub struct GlyphBounds {
    pub x_min: i16,
    pub y_min: i16,
    pub x_max: i16,
    pub y_max: i16,
}


#[derive(Default, Debug, Clone)]
pub struct Glyph {
    pub contours: Vec<Vec<GlyphPoint>>,
    pub bounds: GlyphBounds,
}

