use std::cmp;


#[derive(Default, Debug, Clone)]
pub struct FontUserOptions {
   pub path: String,
   pub font_metric: Option<FontMetric>,
} 


#[derive(Default, Debug, Clone, Copy)]
pub struct FontMetric {
    pub units_per_em: u16,
    pub long_loca: bool,
    pub layout_info: LayoutFontInfo,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct LayoutFontInfo {
    pub num_of_metrics: u16,
    pub ascent: i16,
    pub descent: i16,
    pub line_gap: i16,
} 


#[derive(Default, Debug, Clone, Copy)]
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

impl Glyph {

    pub fn get_edges(&self) -> Vec<(GlyphPoint, GlyphPoint)> {
        let mut ret: Vec<(GlyphPoint, GlyphPoint)> = Vec::new(); 

        for contour in &self.contours {
            for i in 0..contour.len() {
                
                let start = contour[i];
                let end = contour[(i+1) % contour.len()];
                ret.push((start,end));
            }
        }


        ret
    }
    
}
