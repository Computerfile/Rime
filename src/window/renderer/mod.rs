use std::sync::Arc;
use wgpu::{BufferUsages, RenderPipeline, SurfaceConfiguration, VertexAttribute, VertexBufferLayout, VertexStepMode, util::{BufferInitDescriptor, DeviceExt}};

use crate::{ttf::font::Glyph, window::{Point, text_engine::RasterizedGlyph}};

#[derive(Clone, Copy)]
pub enum RenderMode {
    Bitmap,
    SDF,
    MSDF,
}

pub struct Renderer {
    mode: RenderMode,
    device: Arc<wgpu::Device>,
    surface_format: wgpu::TextureFormat,
    points: [Point; 6],
    vertex_buffer: wgpu::Buffer,
    render_pipeline: Option<RenderPipeline>,
    
}


impl Renderer {
    pub fn new(mode: RenderMode, device: Arc<wgpu::Device>, surface_format: wgpu::TextureFormat, config: &SurfaceConfiguration) -> Self {
        
        let points: [Point; 6] = [ 
            Point { x: 0.0, y: 0.0, z: 0.0 },
            Point { x: 0.0, y: -1.0, z: 0.0 },
            Point { x: 1.0, y: 0.0, z: 0.0 },
            Point { x: 1.0, y: -1.0, z: 0.0 },
            Point { x: 1.0, y: 0.0, z: 0.0 },
            Point { x: 0.0, y: -1.0, z: 0.0 },
        ];

        let byte_slice: &[u8] = bytemuck::cast_slice(&points);
        
        let vertex_buffer = BufferInitDescriptor {
            label: Some("LABEL"),
            contents: byte_slice,
            usage: BufferUsages::VERTEX,
        };
        
        let buffer: wgpu::Buffer = device.create_buffer_init(&vertex_buffer);

        let mut ret = Self {
            mode,
            device,
            surface_format,
            points,
            vertex_buffer: buffer,
            render_pipeline: None, 
        };

        let render_pipeline = ret.create_render_pipeline(&ret.device, config);

        ret.render_pipeline = Some(render_pipeline);

        ret
    }

    fn create_render_pipeline(&self, device: &Arc<wgpu::Device>, config: &SurfaceConfiguration) -> RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout Descriptor"),
            bind_group_layouts: &[], 
            immediate_size: 0, 
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/shader.wgsl"));
        
        /* let pipeline_cache = device.create_pipeline_cache(&wgpu::PipelineCacheDescriptor {
            label: Some("Pipeline Cache Descriptor"), 
            fallback: true,
        });
        */

        let vertex_attributes: &[VertexAttribute] = &[
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0, 
                shader_location: 0,
            }
        ]; 

        let vertex_buffer_layout: &[VertexBufferLayout] = &[VertexBufferLayout {
            array_stride: size_of::<Point>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: vertex_attributes,
        }];

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex_entry_point"),
                buffers: vertex_buffer_layout, 
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            layout: Some(&pipeline_layout),
            cache: None,
        });

        render_pipeline

    }


    pub fn draw(&mut self, render_pass: &mut wgpu::RenderPass, glyph: Option<&RasterizedGlyph>) {
        match self.mode {
            RenderMode::Bitmap => {
                // tracing::debug!("pasing glyph: {:?}", glyph);
                // render_pass.set_pipeline(self.render_pipeline.as_ref().unwrap());
                // render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                // render_pass.draw(0..6, 0..1);
            }
            RenderMode::SDF => {
                //println!("SDF {:?}", glyph);
            }
            RenderMode::MSDF => {
                //println!("MSDF {:?}", glyph);
            }

        }
    }

    pub fn render(&mut self, glyph: &Glyph) {

        match self.mode {
            RenderMode::Bitmap => {
                println!("Bitmap {:?}", glyph);
            }
            RenderMode::SDF => {
                println!("SDF {:?}", glyph);
            }
            RenderMode::MSDF => {
                println!("MSDF {:?}", glyph);
            }

        }
    }

    pub fn get_rasterized_from_codepoint(&mut self, codepoint: u32) {


        // let mut r: Rasterizer = Rasterizer::new(letter_a);
        // r.bitmap_rasterizer();

    }


}
