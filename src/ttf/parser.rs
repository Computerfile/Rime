use std::fs::File;

use crate::ttf::font::{FontMetric, FontUserOptions};
use crate::ttf::parser_errors::TableParsingError;

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

// SCHENANIGANS COMMENCE
pub struct TTFParser {
    font_dir: FontDirectory,
    bytes: Vec<u8>,
    font_metric: FontMetric,
} 

trait FromBeBytes: Sized {
    const SIZE: usize;
    fn from_be(bytes: &[u8]) -> Self;
}



impl FromBeBytes for i16 {
    const SIZE: usize = 2;
    fn from_be(bytes: &[u8]) -> Self {
        i16::from_be_bytes(bytes.try_into().unwrap())
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
    pub fn new(font_user_options: FontUserOptions) -> Self {
        let mut ret = Self {
            bytes: read_file(&font_user_options.path),
            font_dir: FontDirectory::default(),
            font_metric: FontMetric::default(),
        }; 

        ret.read_ttf_tables();
        ret.parse_ttf_content();

        ret
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

    fn read_ttf_tables(&mut self) -> Result<(), TableParsingError> {
        self.get_off_sub();
        self.get_tab_dir();
        
        self.debug_print();

        let glyph_id = self.map_glyph_id_codepoint(0x5C)?;
        // 0x0042 -> A (Calibry Format 4)
        // 0x10000 -> A (NotoSansLinearB format 12)

        let byte_offset = self.get_offset_glyph_bytes(glyph_id, self.font_metric.long_loca)?;    
        
        let rasterizing_data = self.get_rasterising_data(byte_offset);

        Ok(()) 
    }
    
    fn get_rasterising_data(&mut self, byte_offsets: (u32, u32)) -> Result<(), TableParsingError> {
        let glyf_tab_base_addr = self.get_offset_from_tag(b"glyf").ok_or(TableParsingError::MalformedTable)?; 
        
        // header
        let glyph_start = glyf_tab_base_addr + byte_offsets.0;
        let num_of_contours: i16 = self.read_at(glyph_start)?; 
        let xMin: i16 = self.read_at(glyph_start + 2)?; 
        let yMin: i16 = self.read_at(glyph_start + 4)?; 
        let xMax: i16 = self.read_at(glyph_start + 6)?; 
        let yMax: i16 = self.read_at(glyph_start + 8)?;

        if num_of_contours >= 0 {
            // single glyf
            tracing::debug!("single {}", num_of_contours);
            
            let mut endPtsOfContours: Vec<u16> = Vec::new();
        
            for i in 0..num_of_contours {
                let end_point: u16 = self.read_at(glyph_start + 10 + (i as u32 * 2))?;
                endPtsOfContours.push(end_point);
            }


        } else if num_of_contours < 0 {
            // compound glyf 
            tracing::debug!("compound");
        } 


        Ok(())
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

        tracing::debug!("{:#?}", self.font_metric);
        
    }


    pub fn get_offset_from_tag(&mut self, tag: &[u8; 4]) -> Option<u32> {
        self.font_dir.table_dir.iter().find(|t| &t.tag == tag).map(|t| t.offset)
    }

    pub fn get_off_sub(&mut self) {
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
    }


    pub fn get_tab_dir(&mut self) {
        let mut n_tab_dir: Vec<TableDirectory> = Vec::new();
        for i in 0..self.font_dir.off_sub.num_tables as usize {
            let entry_start = 12 + (i * 16);
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
    }

    pub fn create_font_metrics(&mut self) -> Result<FontMetric, Box<dyn std::error::Error>>{

        let units_per_em: u16 = self.read_table_field(b"head", 18)?;
        
        let index_to_loc_format: i16 = self.read_table_field(b"head", 50)?;
        let long_loca = index_to_loc_format != 0;

        Ok(FontMetric { units_per_em, long_loca })

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
            return Err(TableParsingError::MalformedTable);
        }; 

        let format: u16 = self.read_table_field(b"cmap", subtable_offset)?; 
        let actual_offset = cmap_top_addr + subtable_offset;

        if format != expected_format {
            return Err(TableParsingError::MalformedTable);
        }

        let glyph_id = match format {
            4 => self.parse_format4(actual_offset, codepoint),
            12 => self.parse_format12(actual_offset, codepoint),
            _ => Err(TableParsingError::MalformedTable),
        }?;

        tracing::debug!("Breaking news Calibrì uses format: {format} ");
        tracing::debug!("Breaking news {:X} in calibri font is {:?}", codepoint, glyph_id);

        Ok(glyph_id)
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
                tracing::debug!("idDelta: {}", idDelta);

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

        Err(TableParsingError::MalformedTable)
    }

    fn parse_format12(&mut self, absolute_offset: u32, codepoint: u32) -> Result<u32, TableParsingError> {

        let format_tableH: u16 = self.read_at(absolute_offset)?; 
        // so this needs to double check that the format is in fact 12 and not 4 but I dont know
        // fonts to test this with so it shant be tested if there is a bug, blame the user
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
        

        // No glyphID offset found, tables probably wrong idk
        Err(TableParsingError::MalformedTable)
        
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
}
