mod window;
mod util;
mod ttf;

use std::env;
use window::{init_app, rasterizer::Rasterizer};
use winit::dpi::LogicalSize;
use window::renderer::{Renderer, RenderMode};
use util::unwrap_or_warn;
use tracing::{info, Level};
use tracing_subscriber::{fmt, EnvFilter};

use crate::{ttf::{font::FontUserOptions, parser::{TTFParser, read_file}}, window::renderer};

#[derive(Clone)]
pub struct UserOptions {
    font_size: f32,
    line_height: f32,
    title: String,
    size: LogicalSize<u32>,

    font: FontUserOptions,

    renderer_mode: RenderMode,
}

impl UserOptions {
    pub fn new(
        title: Option<String>, 
        size: Option<LogicalSize<u32>>,
        font_size: Option<f32>, 
        line_height: Option<f32>, 
        font: Option<FontUserOptions>,
        renderer_mode: Option<RenderMode>,
        ) -> Self {
        
        
        Self {
            title: unwrap_or_warn(title, "Rime".to_string(), "no title provided, using default"),
            size: unwrap_or_warn(size, LogicalSize::new(800, 600), "no size provided, using default"),
            font_size: unwrap_or_warn(font_size, 14.0, "no font size provided, using default"),
            line_height: unwrap_or_warn(line_height, 20.0, "no line height provided, using default"),
            font: font.unwrap_or_else(|| {
                eprintln!("No Font Provided, using default");

                let fallback_font_path: String = match env::var("RIME_FONT_PATH") {
                    Ok(val) => val,
                    Err(env::VarError::NotPresent) => { 
                        panic!("Value Not Found");
                    },
                    Err(env::VarError::NotUnicode(raw)) => {
                        panic!("Can't Parse default font ENV value propperly")
                    }
                };
                // /System/Library/Fonts/Supplemental/NotoSansLinearB-Regular.ttf
                FontUserOptions { path: fallback_font_path, font_metric: None }
            }),
            renderer_mode: unwrap_or_warn(renderer_mode, RenderMode::Bitmap, "No RenderMode Provided Defaulting to Bitmap")
        }

    }
}

impl Default for UserOptions {
    fn default() -> Self {
        Self::new(None, None, None, None, None, None)
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    // INIT LOGGING
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
    // DEFAULTS
    let user_options = UserOptions::default();
    
    // TTF Parser
        // read_file();
    // let mut parser = TTFParser::new(&user_options.font);
    
    // let letter_a = parser.fetch_char_from_cache(0x42)?;
    
    // tracing::debug!("{:?}", letter_a);


    init_app(&user_options); 
    Ok(())
}
