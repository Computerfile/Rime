use std::{fmt::Display, error::Error};

#[derive(Debug)]
pub enum TableParsingError {
    MalformedTable,
    UnsupportedFormat(u16),
    OffsetOutOfBounds { offset: u32, table_len: usize },
}


impl Display for TableParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TableParsingError::MalformedTable => write!(f, "table is malformed"),
            TableParsingError::UnsupportedFormat(fmt) => write!(f, "unsupported subtable format: {fmt}"),
            TableParsingError::OffsetOutOfBounds { offset, table_len } => {
                write!(f, "offset {offset} is out of bounds for table of length {table_len}")
            }
        } 
    }

} 


impl Error for TableParsingError{ }
