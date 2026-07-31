use std::sync::Arc;
use wgpu::{BindGroup, BindGroupLayout, BufferUsages, Origin3d, RenderPipeline, SurfaceConfiguration, TextureAspect, TextureDescriptor, TextureDimension, TextureView, TextureViewDescriptor, VertexAttribute, VertexBufferLayout, VertexStepMode, util::{BufferInitDescriptor, DeviceExt}};

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
    texture: Option<wgpu::Texture>,
    texture_view: Option<wgpu::TextureView>,
    sampler: wgpu::Sampler,
    bind_group_layout: Option<BindGroupLayout>,
    bind_group: Option<BindGroup>
}



pub fn create_quad_for_char( x: f32, y: f32, atlas_cols: f32, atlas_rows: f32, char_grid_x: f32, char_grid_y: f32) -> [Point; 6] {

 //     let u_min = char_grid_x / atlas_cols;
 //     let v_min = char_grid_y / atlas_rows;
 //     let u_max = (char_grid_x + 1.0) / atlas_cols;
 //     let v_max = (char_grid_y + 1.0) / atlas_rows;
 
    let u_min = 0.0;
    let v_min = 0.0;
    let u_max = 1.0;
    let v_max = 1.0;
 
    
    let p1 = Point::new(x, y, 0.0, u_min, v_min); 
    let p2 = Point::new(x, y - 1.0, 0.0, u_min, v_max); 
    let p3 = Point::new(x + 1.0, y, 0.0, u_max, v_min); 

    let p4 = Point::new(x + 1.0, y, 0.0, u_max, v_min); 
    let p5 = Point::new(x, y - 1.0, 0.0, u_min, v_max); 
    let p6 = Point::new(x + 1.0, y - 1.0, 0.0, u_max, v_max); 

    [p1, p2, p3, p4, p5, p6]
}

impl Renderer {
    pub fn new(mode: RenderMode, device: Arc<wgpu::Device>, surface_format: wgpu::TextureFormat, config: &SurfaceConfiguration) -> Self {
        
        let points: [Point; 6] =  create_quad_for_char(0.0, 0.5, 16.0, 16.0, 0.0, 0.0); 

        let byte_slice: &[u8] = bytemuck::cast_slice(&points);
        
        let vertex_buffer = BufferInitDescriptor {
            label: Some("LABEL"),
            contents: byte_slice,
            usage: BufferUsages::VERTEX,
        };
        
        let buffer: wgpu::Buffer = device.create_buffer_init(&vertex_buffer);


        let sampler: wgpu::Sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let mut ret = Self {
            mode,
            device,
            surface_format,
            points,
            vertex_buffer: buffer,
            render_pipeline: None, 
            texture: None,
            texture_view: None,
            sampler,
            bind_group_layout: None,
            bind_group: None,
        };


        ret.bind_group_layout = Some(ret.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("glyph bind group layout"),
            entries: &[

                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        }));


        let render_pipeline = ret.create_render_pipeline(&ret.device, config);

        ret.render_pipeline = Some(render_pipeline);

        ret
    }

    fn create_render_pipeline(&self, device: &Arc<wgpu::Device>, config: &SurfaceConfiguration) -> RenderPipeline {

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout Descriptor"),
            bind_group_layouts: &[self.bind_group_layout.as_ref()], 
            immediate_size: 0, 
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/bitmap_shader.wgsl"));
        
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
            },
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 12, 
                shader_location: 1,
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
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
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


    pub fn draw(&mut self, render_pass: &mut wgpu::RenderPass, glyph: Option<&RasterizedGlyph>, queue: &wgpu::Queue) {
        match self.mode {
            RenderMode::Bitmap => {
                
                if let Some(g) = glyph {

                    let size = wgpu::Extent3d {
                        width: g.width,
                        height: g.height,
                        depth_or_array_layers: 1
                    };

                    let data_layout = wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(g.width), 
                        rows_per_image: Some(g.height), 

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

                    queue.write_texture(*texel_texture_info, &g.data, data_layout, size);

                    self.texture_view = Some(self.texture.as_ref().unwrap().create_view(&TextureViewDescriptor::default()));

                    self.bind_group = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("glyph bind group"),
                        layout: &self.bind_group_layout.as_ref().unwrap(),
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
                    
                    // tracing::debug!("pasing glyph: {:?}", glyph);
                    render_pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
                    render_pass.set_pipeline(self.render_pipeline.as_ref().unwrap());
                    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                    render_pass.draw(0..6, 0..1);
                }

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
