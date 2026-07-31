use std::{fmt::Display, error::Error};

#[derive(Debug)]
pub enum TableParsingError {
    MalformedTable,
    UnsupportedFormat(u16),
    OffsetOutOfBounds { offset: u32, table_len: usize },
    GlyphIdOutOfRange { glyph_id: u32, num_glyphs: u16 },
    CompositeDepthExceeded { count: u32, max: u32 },
} 


impl Display for TableParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TableParsingError::MalformedTable => write!(f, "table is malformed"),
            TableParsingError::UnsupportedFormat(fmt) => write!(f, "unsupported subtable format: {fmt}"),
            TableParsingError::OffsetOutOfBounds { offset, table_len } => {
                write!(f, "offset {offset} is out of bounds for table of length {table_len}")
            }
            TableParsingError::CompositeDepthExceeded { count, max} => {
                write!(f, "recursion depth {count} exceeded max of {max}")
            }
            TableParsingError::GlyphIdOutOfRange{ glyph_id, num_glyphs } => {
                write!(f, "offset {glyph_id} is out of bounds for table of length {num_glyphs}")
            }

        } 
    }

} 


impl Error for TableParsingError{ }

