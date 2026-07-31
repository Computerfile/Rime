use crate::{UserOptions, ttf::font::{Glyph, GlyphPoint}, window::text_engine::RasterizedGlyph};


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
        let raw_width: i16 = (self.glyph.bounds.x_max - self.glyph.bounds.x_min);
        let raw_height: i16 = (self.glyph.bounds.y_max - self.glyph.bounds.y_min);

        let units_per_em = self.user_options.font.font_metric.unwrap().units_per_em;
        let font_size = self.user_options.font_size;

        let width = ((raw_width as f32 / units_per_em as f32) * font_size).ceil() as usize;
        let height = ((raw_height as f32 / units_per_em as f32) * font_size).ceil() as usize;

        let mut bitmap_data: Vec<u8> = vec![0; (width as usize) * (height as usize)];
        let edges: Vec<((f32, f32), (f32, f32))> = self.glyph.get_edges()
            .into_iter()
            .map(|(a, b)| {
                (
                    self.to_pixel_space(a, self.glyph.bounds.x_min, self.glyph.bounds.y_min, units_per_em, font_size, height),
                    self.to_pixel_space(b, self.glyph.bounds.x_min, self.glyph.bounds.y_min, units_per_em, font_size, height),
                )
            })
            .collect();

        for i in 0..height {
            let mut crossings: Vec<i16> = vec![];
        let i_offset: f32 = i as f32 + 0.5; 

        for edge in &edges {

            if (edge.0.1 < i_offset && edge.1.1 > i_offset ) || (edge.0.1 > i_offset && edge.1.1 < i_offset) {
                let proportional_distance = ((i_offset -edge.0.1)) / ((edge.1.1 - edge.0.1) as f32);
                    let crossing_x = edge.0.0 as f32 + proportional_distance * (edge.1.0 - edge.0.0) as f32;
                    crossings.push(crossing_x.clamp(0.0, width as f32) as i16);

                }

            }
            
            crossings.sort();
            for pair in crossings.chunks(2) {
                if let [left, right] = pair {
                    for x in *left..*right {
                        let index = (i * width + x as usize);
                        bitmap_data[index] = 255;
                    }
                }
            }

        } 


        for row in 0..height {
            let mut line = String::new();
            for col in 0..width {
                let px = bitmap_data[row * width + col];
                line.push(if px > 0 { '#' } else { '.' });
            }
            println!("{}", line)
        }

        RasterizedGlyph {
            data: bitmap_data,        
            width: width as u32,
            height: height as u32
        }
    }


    fn to_pixel_space(&self, point: GlyphPoint, x_min: i16, y_min: i16, units_per_em: u16, font_size: f32, height: usize) -> (f32, f32) {
        let shifted_x = (point.x - x_min) as f32;
        let shifted_y = (point.y - y_min) as f32;
        
        let scale = font_size / units_per_em as f32;
        let px = shifted_x * scale;
        let py = shifted_y * scale;
        
        let flipped_y = height as f32 - py;
        
        (px, flipped_y)
    }



}



