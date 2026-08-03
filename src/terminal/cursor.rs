use std::sync::Arc;

use wgpu::{RenderPipeline, SurfaceConfiguration};

pub struct Cursor {
    pub x: u32,
    pub y: u32,
}


impl Cursor {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
        }
    }
    

    pub fn create_render_pipeline(&self, device: &Arc<wgpu::Device>, config: &SurfaceConfiguration) -> Option<RenderPipeline> {
        None
    }

}
