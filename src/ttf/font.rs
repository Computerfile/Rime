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



pub struct GlyphPoint {
    x: i16, 
    y: i16,
    on_curve: bool,
}

pub struct Glyph {
    contours: Vec<Vec<GlyphPoint>>,
}

