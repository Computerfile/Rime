use std::collections::HashMap;
use std::{mem};

use tracing::warn;

use crate::ttf::font::{FontMetric, FontUserOptions, Glyph, GlyphBounds, GlyphPoint, LayoutFontInfo};
use crate::ttf::parser_errors::TableParsingError;

const FALLBACK_GLYPH_ID: u32 = 0; 

pub fn read_file(file: &str) -> Vec<u8> {
   let bytes: Vec<u8> = match std::fs::read(file) {
        Ok(buf) => buf,
        Err(e) => {
            tracing::error!("Error Reading ttf file {:?} {:?} ", file, e);
            return Vec::new();
        }
   };

   bytes
}


#[derive(Debug, Default)]
pub struct OffsetSubtable {
    scaler_type: u32,
	num_tables: u16,
	search_range: u16,
	entry_selector: u16,
	range_shift: u16,
}


#[derive(Debug, Default, Clone)]
pub struct TableDirectory {
    pub tag: [u8; 4],
	pub checkSum: u32,
	pub offset: u32,
	pub length: u32,
}


#[derive(Debug, Default)]
pub struct FontDirectory {
    off_sub: OffsetSubtable,
    table_dir: Vec<TableDirectory>,
}

struct Format4Subtable {
    end_code: Vec<u16>,
    start_code: Vec<u16>,
    id_delta: Vec<i16>,
    id_range_offset: Vec<u16>,
    glyph_id_array: Vec<u16>,
}

struct Format12Subtable {
    groups: Vec<(u32, u32, u32)>,
}

pub struct TTFParser {
    font_dir: FontDirectory,
    bytes: Vec<u8>,
    pub font_metric: FontMetric,
    pub glyph_cache: HashMap<u32, Glyph>, 
    pub font_limits: FontLimits,
    
} 

#[derive(Default)]
pub struct FontLimits {
    num_glyphs: u16,
    recursion_limits: RecursionLimits,
    pub h_metrics: HorizontalMetrics,
}

#[derive(Default)]
pub struct HorizontalMetrics {
    advanced_widths: HashMap<u32, u16>, 
    left_side_bearings: HashMap<u32, i16>, 
} 

#[derive(Default)]
pub struct RecursionLimits {
    max_component_element: u16,
    max_component_depth: u16,
}

trait FromBeBytes: Sized {
    const SIZE: usize;
    fn from_be(bytes: &[u8]) -> Self;
}


impl FromBeBytes for i8 {
    const SIZE: usize = 1;
    fn from_be(bytes: &[u8]) -> Self {
        i8::from_be_bytes(bytes.try_into().unwrap())
    }
}


impl FromBeBytes for i16 {
    const SIZE: usize = 2;
    fn from_be(bytes: &[u8]) -> Self {
        i16::from_be_bytes(bytes.try_into().unwrap())
    }
}


impl FromBeBytes for u8 {
    const SIZE: usize = 1;
    fn from_be(bytes: &[u8]) -> Self {
        u8::from_be_bytes(bytes.try_into().unwrap())
    }
}


impl FromBeBytes for u16 {
    const SIZE: usize = 2;
    fn from_be(bytes: &[u8]) -> Self {
        u16::from_be_bytes(bytes.try_into().unwrap())
    }
}

impl FromBeBytes for u32 {
    const SIZE: usize = 4;
    fn from_be(bytes: &[u8]) -> Self {
        u32::from_be_bytes(bytes.try_into().unwrap())
    }
}

impl TTFParser {
    pub fn new(font_user_options: &FontUserOptions) -> Result<Self, TableParsingError> {
        let mut ret = Self {
            bytes: read_file(&font_user_options.path),
            font_dir: FontDirectory::default(),
            font_metric: FontMetric::default(),
            glyph_cache: HashMap::default(),
            font_limits: FontLimits::default(),
        }; 
        
        
        ret.get_off_sub()?;
        ret.get_tab_dir()?;

        ret.parse_ttf_content();

        let font_limits: FontLimits = match ret.get_memory_requirements() {
            Ok(val) => val,
            Err(err) => FontLimits::default(), 
        };

        ret.font_limits = font_limits;

        Ok(ret)
    }


    pub fn fetch_char_from_cache(&mut self, codepoint: u32) -> Result<Glyph, TableParsingError> {
        println!("codepoint: {}", codepoint);

        if self.glyph_cache.contains_key(&codepoint) {
            let glyph_data = self.glyph_cache.get(&codepoint).unwrap();
            Ok(glyph_data.clone())
        }else {
            let glyph_data = self.read_ttf_tables(codepoint)?;

            self.glyph_cache.insert(codepoint, glyph_data);

            Ok(self.glyph_cache.get(&codepoint).unwrap().clone())
        }
        
    }

    fn read_at<T: FromBeBytes>(&self, absolute_offset: u32) -> Result<T, TableParsingError> {
        let start = absolute_offset as usize;
        let end = start + T::SIZE;
        if end > self.bytes.len() {
            return Err(TableParsingError::OffsetOutOfBounds { offset: absolute_offset, table_len: self.bytes.len() });
        }
        Ok(T::from_be(&self.bytes[start..end]))
    }

    fn read_table_field<T: FromBeBytes>(&mut self, tag: &[u8; 4], field_offset: u32) -> Result<T, TableParsingError> {
        let table_offset = self
            .get_offset_from_tag(tag)
            .ok_or(TableParsingError::MalformedTable)?;

        let start = (table_offset + field_offset) as usize;
        let end = start + T::SIZE;

        if end > self.bytes.len() {
            return Err(TableParsingError::OffsetOutOfBounds { offset: start as u32, table_len: self.bytes.len() });
        }

        Ok(T::from_be(&self.bytes[start..end]))
    }

    fn read_ttf_tables(&mut self, codepoint: u32) -> Result<Glyph, TableParsingError> {
        
        self.debug_print();

        let glyph_id = self.map_glyph_id_codepoint(codepoint)?;
        
        let g_data = self.map_id_to_glyph_data(glyph_id, codepoint, None)?;
        
        Ok(g_data)
        
    }
    

    fn map_id_to_glyph_data(&mut self, glyph_id: u32, codepoint: u32, depth_val: Option<u32>) -> Result<Glyph, TableParsingError> {
        let depth = depth_val.unwrap_or_else(|| 0); 
        
        if depth > self.font_limits.recursion_limits.max_component_depth as u32 {
            return Err(TableParsingError::CompositeDepthExceeded { count: depth, max: self.font_limits.recursion_limits.max_component_depth as u32 });
        }

        if !self.font_limits.h_metrics.advanced_widths.contains_key(&codepoint) {
            self.parse_hmtx_table(glyph_id, codepoint)?;
        }

        let byte_offset = self.get_offset_glyph_bytes(glyph_id, self.font_metric.long_loca)?;    
        
        let rasterizing_data = self.get_rasterising_data(byte_offset, codepoint, depth)?;
        
        // tracing::debug!("Hello rasterizing_data here {:?}", rasterizing_data);

        Ok(rasterizing_data) 

    }

    fn get_rasterising_data(&mut self, byte_offsets: (u32, u32), codepoint: u32, depth: u32) -> Result<Glyph, TableParsingError> {
        let glyf_tab_base_addr = self.get_offset_from_tag(b"glyf").ok_or(TableParsingError::MalformedTable)?; 
        
        // header
        let glyph_start = glyf_tab_base_addr + byte_offsets.0;
        println!("{:?}", byte_offsets);
        let num_of_contours: i16 = self.read_at(glyph_start)?; 
        let x_min: i16 = self.read_at(glyph_start + 2)?; 
        let y_min: i16 = self.read_at(glyph_start + 4)?; 
        let x_max: i16 = self.read_at(glyph_start + 6)?; 
        let y_max: i16 = self.read_at(glyph_start + 8)?;

        let glyph_bounds = GlyphBounds {
            x_min,
            x_max,
            y_min,
            y_max 
        };

        if num_of_contours >= 0 {
            // single glyf
            tracing::debug!("single glyf {:?}", num_of_contours);
            
            let mut endPtsOfContours: Vec<u16> = Vec::new();
        
            for i in 0..num_of_contours {
                let end_point: u16 = self.read_at(glyph_start + 10 + (i as u32 * 2))?;
                endPtsOfContours.push(end_point);
            }

            let instruction_len_addr = glyph_start + 10 + (num_of_contours as u32 * 2);
            let instruction_len: u16 = self.read_at(instruction_len_addr)?;
            let mut instructions: Vec<u8> = Vec::new();

            for i in 0..instruction_len {
                let instruction_addr = instruction_len_addr + 2 + (i as u32);
                let instruction: u8 = self.read_at(instruction_addr)?;
                instructions.push(instruction);
            }

            let flag_lenght = *endPtsOfContours.last().unwrap_or(&0) as u32 + 1;
            let flag_base_addr = instruction_len_addr + 2 + (instruction_len as u32);
            let mut flags: Vec<u8> = Vec::new();
            let mut flag_mark: u32 = flag_base_addr;

            while flags.len() < flag_lenght as usize {
                let flag: u8 = self.read_at(flag_mark)?;
                let is_repeat_set = (flag >> 3) & 1 == 1;
                
                if is_repeat_set {
                    let repeat_count: u8 = self.read_at(flag_mark + 1)?;

                    for _ in 0..repeat_count+1 {
                        flags.push(flag);
                    }
                    flag_mark += 2;
                }else {
                    flags.push(flag);
                    flag_mark += 1;
                }

            }


            let (mut x_coordinates, x_end_addr): (Vec<i16>, u32) = self.parse_glyf_coordinates(&flags, flag_mark, 1, 4)?;
            let (mut y_coordinates, y_end_addr): (Vec<i16>, u32) = self.parse_glyf_coordinates(&flags, x_end_addr, 2, 5)?;

            let mut final_x_delta: Vec<i16> = Vec::new();
            let mut final_y_delta: Vec<i16> = Vec::new();

            x_coordinates.into_iter().fold(0, |acc, val| {
                let ret = val + acc;
                final_x_delta.push(ret);  
                ret
            });

            y_coordinates.into_iter().fold(0, |acc, val| {
                let ret = val + acc;
                final_y_delta.push(ret);  
                ret
            });

            let glyph: Glyph = self.construct_glyph(&flags, &final_x_delta, &final_y_delta, &endPtsOfContours, glyph_bounds)?;

            Ok(glyph)
        } else {
            // compound glyf 
            tracing::debug!("compound");
            let glyph: Glyph = self.parse_compound_glyph(glyph_start + 10, codepoint, depth, glyph_bounds)?;
            Ok(glyph)
        } 

    }
  
    fn construct_glyph(&mut self, 
        flags: &Vec<u8>, 
        x_deltas: &Vec<i16>, 
        y_deltas: &Vec<i16>, 
        end_pts_of_contours: &Vec<u16>,
        glyph_bounds: GlyphBounds
        ) -> Result<Glyph, TableParsingError>{

        let mut contours: Vec<Vec<GlyphPoint>> = Vec::new();
        let mut current_contour: Vec<GlyphPoint> = Vec::new(); 
        let mut contour_index: usize = 0;

        for i in 0..flags.len() {
            let mut point = GlyphPoint::default();

            let flag: u8 = *flags.get(i).ok_or(TableParsingError::MalformedTable)?;
            let on_curve = (flag >> 0) & 1 == 1;
            let x = x_deltas[i];
            let y = y_deltas[i];
            
            point = GlyphPoint { x, y, on_curve };
            current_contour.push(point);

            let current_contour_point: usize = end_pts_of_contours[contour_index] as usize;
            if i == current_contour_point {
                let c = mem::take(&mut current_contour);
                contours.push(c);
                contour_index+=1;
            }

        }

        let mut glyph: Glyph = Glyph { 
            contours: contours,
            bounds: glyph_bounds,
        };
        Ok(glyph)
    }


    fn parse_compound_glyph(&mut self, glyph_start: u32, codepoint: u32, depth: u32, glyph_bounds: GlyphBounds) -> Result<Glyph, TableParsingError> {
            
        let mut glyph = Glyph { contours: Default::default(), bounds: glyph_bounds };
        let mut cursor = glyph_start; 
        let mut i = 0;
        let mut MORE_COMPONENTS = true;

        while MORE_COMPONENTS {
            let flags: u16 = self.read_at(cursor)?; 
            cursor += 2;
            let mut glyph_index: u16 = self.read_at(cursor)?;
            cursor += 2;
            
             
            let ARG_1_AND_2_ARE_WORDS = self.get_bit(flags, 0);
            let ARGS_ARE_XY_VALUES = self.get_bit(flags, 1);
            let ROUND_XY_TO_GRID = self.get_bit(flags, 2);
            let WE_HAVE_A_SCALE = self.get_bit(flags, 3);
            MORE_COMPONENTS = self.get_bit(flags, 5); 
            let WE_HAVE_AN_X_AND_Y_SCALE = self.get_bit(flags, 6);
            let WE_HAVE_A_TWO_BY_TWO = self.get_bit(flags, 7);
            let WE_HAVE_INSTRUCTIONS = self.get_bit(flags, 8);
            let USE_MY_METRICS = self.get_bit(flags, 9);
            let OVERLAP_COMPOUND = self.get_bit(flags, 10);
           
            let (argument_1, argument_2): (i32, i32) = if ARG_1_AND_2_ARE_WORDS {
                let a1: i16 = self.read_at(cursor)?;
                let a2: i16 = self.read_at(cursor + 2)?;
                cursor = cursor + 4;
                (a1 as i32, a2 as i32)
            } else {
                let a1: i8 = self.read_at(cursor)?;
                let a2: i8 = self.read_at(cursor + 1)?;
                cursor = cursor + 2;
                (a1 as i32, a2 as i32)
            };

            if WE_HAVE_A_SCALE {
                cursor += 2;
            } 

            if WE_HAVE_AN_X_AND_Y_SCALE {
                cursor += 4;
            } 

            if WE_HAVE_A_TWO_BY_TWO {
                cursor += 8;
            }


            if !ARGS_ARE_XY_VALUES {
                tracing::warn!("point-matching mode encountered, not yet supported, component placement may be approximate/skipped");
                glyph_index = 0;
            }



            let mut sub_glyph = self.map_id_to_glyph_data(glyph_index as u32, codepoint, Some(depth+1))?;

            for countour in sub_glyph.contours.iter_mut() {
                for point in countour.iter_mut() {
                    let new_x = (point.x as i32 + argument_1).clamp(i16::MIN as i32, i16::MAX as i32);
                    let new_y = (point.y as i32 + argument_2).clamp(i16::MIN as i32, i16::MAX as i32);

                    point.x = new_x as i16;
                    point.y = new_y as i16;
                } 
            }

            glyph.contours.extend(sub_glyph.contours);

            if self.font_limits.recursion_limits.max_component_element < i {
                return Err(TableParsingError::CompositeDepthExceeded { count: i as u32, max: self.font_limits.recursion_limits.max_component_element as u32});
            }

            i = i+1;
            tracing::debug!("component {}: glyph_index={}, args=({}, {}), more={}", i, glyph_index, argument_1, argument_2, MORE_COMPONENTS);
        }

        Ok(glyph)
    }

    fn parse_glyf_coordinates(&mut self, flags: &Vec<u8>, start_address: u32, short_vector: u8, is_same: u8) -> Result<(Vec<i16>, u32), TableParsingError> {
        let mut coordinate_vec: Vec<i16> = Vec::new(); 
        let mut cursor: u32 = start_address;
        for i in 0..flags.len() {
            let flag: u8 = *flags.get(i).ok_or(TableParsingError::MalformedTable)?;
            let is_x_short_vector = (flag >> short_vector) & 1 == 1;
            let is_x_same = (flag >> is_same) & 1 == 1;
            
            let (byte_count, delta): (u32, i16) = match (is_x_short_vector, is_x_same) {
                (true, true)   => { 
                    let val: u8 = self.read_at(cursor)?;
                    (1 as u32, val as i16)
                },
                (true, false)  => { 
                    let val: u8 = self.read_at(cursor)?;
                    (1 as u32, (-1 * val as i16))
                },
                (false, true)  => { 
                    (0 as u32, 0 as i16)
                },
                (false, false) => { 
                    let val: i16 = self.read_at(cursor)?;
                    (2 as u32, val as i16)
                },
            };

            coordinate_vec.push(delta);

            cursor += byte_count;
        }

        Ok((coordinate_vec, cursor))
    }


    fn get_offset_glyph_bytes(&mut self, glyph_id: u32, long: bool) -> Result<(u32, u32), TableParsingError> {
        let loca_base = self.get_offset_from_tag(b"loca").ok_or(TableParsingError::MalformedTable)?;

        let short_format_offset: u32 = loca_base + (glyph_id * 2);
        let long_format_offset: u32 = loca_base + (glyph_id * 4);
        
        let offset: u32 = if long { long_format_offset }else { short_format_offset };

        let (start_offset, end_offset): (u32, u32) = if long {
            (self.read_at(offset)?, self.read_at(offset + 4)?)
        } else {
            let b1: u16 = self.read_at(offset)?;
            let b2: u16 = self.read_at(offset + 2)?;
            (b1 as u32 * 2, b2 as u32 * 2)
        };

        Ok((start_offset, end_offset))    
    }


    fn parse_ttf_content(&mut self) {

        self.font_metric = match self.create_font_metrics() {
            Ok(val) => val,
            Err(e) => {
                tracing::error!("Error getting font_metric: {:?}", e);
                FontMetric::default()
            },
        };

        // tracing::debug!("{:#?}", self.font_metric);
        
    }

    fn parse_hmtx_table(&mut self, glyph_id: u32, codepoint: u32) -> Result<(), TableParsingError> {

        let num_metrics: u16 = self.font_metric.layout_info.num_of_metrics;
        
        let mut aw_offset  = 0;

        if glyph_id < (num_metrics as u32) {
            aw_offset = glyph_id * 4; 

            let lsb: i16 = self.read_table_field(b"hmtx",  aw_offset + 2)?;
            self.font_limits.h_metrics.left_side_bearings.insert(codepoint, lsb);

        }else {
            if num_metrics == 0 {
                return Err(TableParsingError::OffsetOutOfBounds { 
                    offset: 0, 
                    table_len: 0, 
                });
            }

            aw_offset = (num_metrics as u32 - 1) * 4;
            let lsb_offset: u32 = ((num_metrics as u32) * 4) + (glyph_id - num_metrics as u32);

            let lsb: i16 = self.read_table_field(b"hmtx",  lsb_offset)?;
            self.font_limits.h_metrics.left_side_bearings.insert(codepoint, lsb);
        }
    
        let aw: u16 = self.read_table_field(b"hmtx", aw_offset)?;
        
        self.font_limits.h_metrics.advanced_widths.insert(codepoint, aw);

        Ok(())
    }


    pub fn get_offset_from_tag(&mut self, tag: &[u8; 4]) -> Option<u32> {
        self.font_dir.table_dir.iter().find(|t| &t.tag == tag).map(|t| t.offset)
    }

    pub fn get_off_sub(&mut self) -> Result<(), TableParsingError> {
        
        if self.bytes.len() < 12 {
            return Err(TableParsingError::OffsetOutOfBounds { offset: 20, table_len: self.bytes.len() })
        }

        self.font_dir.off_sub.scaler_type = u32::from_be_bytes([
                                 self.bytes[0], 
                                 self.bytes[1], 
                                 self.bytes[2], 
                                 self.bytes[3]
                            ]);
        self.font_dir.off_sub.num_tables = u16::from_be_bytes([
                self.bytes[4], 
                self.bytes[5],
            ]);

        Ok(())
    }


    pub fn get_tab_dir(&mut self) -> Result<(), TableParsingError> {
        let mut n_tab_dir: Vec<TableDirectory> = Vec::new();
        for i in 0..self.font_dir.off_sub.num_tables as usize {
            let entry_start = 12 + (i * 16);
            
            if self.bytes.len() < entry_start + 16 {
                return Err(TableParsingError::OffsetOutOfBounds { offset: (entry_start + 16) as u32, table_len: self.bytes.len() });
            }

            let slice = &self.bytes[entry_start..entry_start+16];
            let temp_t_dir = TableDirectory {
                tag: slice[0..4].try_into().unwrap(),
                checkSum: u32::from_be_bytes(slice[4..8].try_into().unwrap()),
                offset: u32::from_be_bytes(slice[8..12].try_into().unwrap()),
                length: u32::from_be_bytes(slice[12..16].try_into().unwrap()),
            };
            n_tab_dir.push(temp_t_dir);
        } 
        self.font_dir.table_dir = n_tab_dir;
        Ok(())
    }

    pub fn create_font_metrics(&mut self) -> Result<FontMetric, Box<dyn std::error::Error>>{

        let units_per_em: u16 = self.read_table_field(b"head", 18)?;
        
        let index_to_loc_format: i16 = self.read_table_field(b"head", 50)?;
        let long_loca = index_to_loc_format != 0;

        let numOfHMetrics: u16 = self.read_table_field(b"hhea", 34)?;
        let ascent: i16 = self.read_table_field(b"hhea", 4)?;
        let descent: i16 = self.read_table_field(b"hhea", 6)?;
        let line_gap: i16 = self.read_table_field(b"hhea", 10)?;
        
        tracing::debug!("{:#?}", numOfHMetrics);


        let fm = FontMetric {
            units_per_em, 
            long_loca, 
            layout_info: LayoutFontInfo {
                num_of_metrics: numOfHMetrics,
                ascent,
                descent,
                line_gap
            },
        };

        Ok(fm)

    }


    // cmap header
    fn map_glyph_id_codepoint(&mut self, codepoint: u32) -> Result<u32, TableParsingError>  {
        let cmap_top_addr = self.get_offset_from_tag(b"cmap").ok_or(TableParsingError::MalformedTable)?;
        let num_tables: u16 = self.read_table_field(b"cmap", 2)?;
        

        let mut format_4_record: Option<u32> = None;  
        let mut format_12_record: Option<u32> = None;  

        for i in 0..num_tables {
            let record_offset = 4 + (i as u32) * 8;
            let platform_id: u16 = self.read_table_field(b"cmap", record_offset)?;
            let encoding_id: u16 = self.read_table_field(b"cmap", record_offset + 2)?;
            let subtable_offset: u32 = self.read_table_field(b"cmap", record_offset + 4)?;

            match (platform_id, encoding_id) {
                // format 12  encodings
                (3, 10) | (0, 4) => { format_12_record = Some(subtable_offset); },
                // format 4 encodings
                (3, 1) | (0, 3) => { format_4_record = Some(subtable_offset); },
                _ => { }
            }

        }

        let (subtable_offset, expected_format) = if let Some(offset) = format_12_record {
            (offset, 12)
        }else if let Some(offset) = format_4_record {
            (offset, 4)
        }else {
            tracing::error!("Unsupported cmap subtable format this font isn't compatible yet, try a different font");
            return Ok(FALLBACK_GLYPH_ID);
        }; 

        let format: u16 = self.read_table_field(b"cmap", subtable_offset)?; 
        let actual_offset = cmap_top_addr + subtable_offset;

        if format != expected_format {
            return Err(TableParsingError::MalformedTable);
        }

        let glyph_id = match format {
            4 => self.parse_format4(actual_offset, codepoint),
            12 => self.parse_format12(actual_offset, codepoint),
            _ => Ok(FALLBACK_GLYPH_ID),
        }?;


        if glyph_id as u16 >= self.font_limits.num_glyphs {
            
            return Err(TableParsingError::GlyphIdOutOfRange { glyph_id, num_glyphs: self.font_limits.num_glyphs });
        }

        Ok(glyph_id)
    }
    

    //maxp header
    pub fn get_memory_requirements(&mut self) -> Result<FontLimits, TableParsingError> {

        let num_glyphs: u16 = self.read_table_field(b"maxp", 4)?;


        let max_component_element: u16 = self.read_table_field(b"maxp", 28)?;
        let max_component_depth: u16 = self.read_table_field(b"maxp", 30)?;

        Ok(FontLimits {
            num_glyphs, 
            recursion_limits: RecursionLimits { max_component_element, max_component_depth },
            h_metrics: Default::default(), 
        })
        
    }


    fn parse_format4(&mut self, absolute_offset: u32, codepoint: u32) -> Result<u32, TableParsingError> {
        // glyphIdArray[ idRangeOffset[i]/2 + (codepoint - startCode[i]) - (segCount - i) ]
        let format_tableH: u16 = self.read_at(absolute_offset)?;
        if(format_tableH != 4) {
            return Err(TableParsingError::MalformedTable)
        }
        
        let seg_count_raw: u16 = self.read_at(absolute_offset + 6)?;
        let seg_count: u32 = seg_count_raw as u32 / 2;

        let total_table_offsets: u32 = 14; 
        

        for i in 0..seg_count {
            let end_code: u16 = self.read_at(absolute_offset + total_table_offsets + (i * 2))?; 
            let start_code: u16 = self.read_at(absolute_offset + total_table_offsets + 2 + (seg_count * 2) + (i*2))?;

            tracing::debug!("segment {i}: start={:#06X} end={:#06X}", start_code, end_code);

            if ((start_code as u32 <= codepoint) && codepoint <= (end_code as u32)) {
                let idRangeOffset: u16 = self.read_at(absolute_offset + total_table_offsets + 2 + (seg_count*6) + (i*2))?;
                let idDelta: i16 = self.read_at(absolute_offset + total_table_offsets + 2 + (seg_count*4) + (i*2))?;

                if(idRangeOffset == 0) {

                    let glyph_id: u32 = (codepoint as i32 + idDelta as i32) as u32 & 0xFFFF;
                    tracing::debug!("codepoint: {} + idDelta: {} = {}", codepoint, idDelta, glyph_id);

                    return Ok(glyph_id);

                }else {

                    let index = idRangeOffset as u32 / 2 + (codepoint - start_code as u32) - (seg_count - i);
                    let glyph_array_start = absolute_offset + total_table_offsets + 2 + seg_count * 8;
                    let raw: u16 = self.read_at(glyph_array_start + index * 2)?;

                    tracing::debug!("idRangeOffset: {}", idRangeOffset);
                    tracing::debug!("index: {}", index);
                    tracing::debug!("raw: {}", raw);

                    if raw == 0 { return Ok(0) }

                    let glyph_id: u32 = (raw as i32 + idDelta as i32) as u32 & 0xFFFF; 

                    tracing::debug!("SECOND codepoint: {} + idDelta: {} = {}", codepoint, idDelta, glyph_id);

                    return Ok(glyph_id);

                }

            }

        }

        Ok(FALLBACK_GLYPH_ID)
    }

    fn parse_format12(&mut self, absolute_offset: u32, codepoint: u32) -> Result<u32, TableParsingError> {

        let format_tableH: u16 = self.read_at(absolute_offset)?; 
        let num_groups: u32 = self.read_at(absolute_offset+12)?; 
        let start_group_offset = 16;
        let size_group_table = 12;
            
        for i in 0..num_groups {
            let current_group_offset = start_group_offset + (i * size_group_table);

            let startCharCode: u32 = self.read_at(absolute_offset + current_group_offset)?; 
            let endCharCode: u32 = self.read_at(absolute_offset + current_group_offset + 4)?; 
            
            if startCharCode <= codepoint && codepoint <= endCharCode {

                let startGlyphID: u32 = self.read_at(absolute_offset + current_group_offset + 8)?;
                return Ok(startGlyphID + (codepoint - startCharCode));
            }
        }
        

        Ok(FALLBACK_GLYPH_ID) 
    }

    pub fn debug_print(&self) {
        tracing::debug!("=== Offset Subtable ===");
        tracing::debug!("scaler_type: 0x{:08X}", self.font_dir.off_sub.scaler_type);
        tracing::debug!("num_tables: {}", self.font_dir.off_sub.num_tables);
        
        tracing::debug!("=== Table Directory ===");
        for t in &self.font_dir.table_dir {
            tracing::debug!("  tag: {} offset: {} length: {}", 
                std::str::from_utf8(&t.tag).unwrap_or("????"),
                t.offset,
                t.length
            );
        }


    }

    fn get_bit(&mut self, value: u16, bit: u8) -> bool {
        (value >> bit) & 1 == 1
    }

}
