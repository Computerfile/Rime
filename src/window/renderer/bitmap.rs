use std::sync::Arc;

use wgpu::{Origin3d, BindGroup, TextureAspect, TextureDescriptor, TextureDimension, TextureViewDescriptor, BindGroupLayout};

use crate::window::{renderer::Renderer, text_engine::RasterizedGlyph};

pub struct BitMapRenderer { 
    device: Arc<wgpu::Device>,
    texture: Option<wgpu::Texture>,
    texture_view: Option<wgpu::TextureView>,
    sampler: wgpu::Sampler,
    bind_group: Option<BindGroup>,
}


impl BitMapRenderer {

    pub fn new(
        device: Arc<wgpu::Device>,
        texture: Option<wgpu::Texture>, 
        texture_view: Option<wgpu::TextureView>, 
        bind_group: Option<BindGroup>) -> Self {

        let sampler: wgpu::Sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            device,
            texture,
            texture_view,
            sampler,
            bind_group
        }
    }

    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass, glyph: &RasterizedGlyph, queue: &wgpu::Queue, bind_group_layout: &BindGroupLayout) -> Result<wgpu::BindGroup, anyhow::Error> {
        let size = wgpu::Extent3d {
            width: glyph.width,
            height: glyph.height,
            depth_or_array_layers: 1
        };

        let data_layout = wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(glyph.width), 
            rows_per_image: Some(glyph.height), 
        };

        self.texture = Some(self.device.create_texture(&TextureDescriptor {
            label: Some("texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        }));

        
        let texel_texture_info = &wgpu::TexelCopyTextureInfoBase {
            texture: self.texture.as_ref().unwrap(),
            mip_level: 0,
            origin: Origin3d {
                x: 0,  
                y: 0, 
                z: 0,
            },
            aspect: TextureAspect::All,    
        };
            
        queue.write_texture(*texel_texture_info, &glyph.data, data_layout, size);
        
        self.texture_view = Some(self.texture.as_ref().unwrap().create_view(&TextureViewDescriptor::default()));


        let bind_group = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("glyph bind group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(self.texture_view.as_ref().unwrap()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        }));
        
    
        Ok(bind_group.unwrap())

    }

}
