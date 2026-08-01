pub mod bitmap; 
use std::sync::Arc;
use wgpu::{BindGroup, BindGroupLayout, BufferUsages, Origin3d, RenderPipeline, SurfaceConfiguration, TextureAspect, TextureDescriptor, TextureDimension, TextureUsages, TextureView, TextureViewDescriptor, VertexAttribute, VertexBufferLayout, VertexStepMode, util::{BufferInitDescriptor, DeviceExt}};
use winit::dpi::PhysicalSize;

use crate::{terminal::{CELL_HEIGHT_PX, CELL_WIDTH_PX, Terminal, cell::{Cell, CellInstance, CellState}}, ttf::font::Glyph, window::{Point, glyph_cache::{self, AtlasRect, GlyphCache}, renderer::bitmap::BitMapRenderer, text_engine::{RasterizedGlyph, TextEngine}}};

#[derive(Clone, Copy, PartialEq)]
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
    pub instance_buffer: wgpu::Buffer,
    render_pipeline: Option<RenderPipeline>,
    /*texture: Option<wgpu::Texture>,
    texture_view: Option<wgpu::TextureView>,
    sampler: wgpu::Sampler,
    bind_group: Option<BindGroup>,*/

    bind_group_layout: Option<BindGroupLayout>,

    bitmap_renderer: Option<BitMapRenderer>,

    pub cell_instance_count: u32, 
    glyph_cache: GlyphCache,
    glyph_texture: wgpu::Texture,
    size: PhysicalSize<u32>,

    glyph_bind_group: BindGroup,
}



pub fn create_quad_for_char( x: f32, y: f32, atlas_cols: f32, atlas_rows: f32, char_grid_x: f32, char_grid_y: f32, width: u32, height: u32) -> [Point; 6] {

 //     let u_min = char_grid_x / atlas_cols;
 //     let v_min = char_grid_y / atlas_rows;
 //     let u_max = (char_grid_x + 1.0) / atlas_cols;
 //     let v_max = (char_grid_y + 1.0) / atlas_rows;


    let cell_width_ndc = (CELL_WIDTH_PX * 2.0) / width as f32;
    let cell_height_ndc = (CELL_HEIGHT_PX * 2.0) / height as f32; 

    let u_min = 0.0;
    let v_min = 0.0;
    let u_max = 1.0;
    let v_max = 1.0;
 
    
    let p1 = Point::new(x, y, 0.0, u_min, v_min);
    let p2 = Point::new(x, y - cell_height_ndc, 0.0, u_min, v_max);
    let p3 = Point::new(x + cell_width_ndc, y, 0.0, u_max, v_min);
    let p4 = Point::new(x + cell_width_ndc, y, 0.0, u_max, v_min);
    let p5 = Point::new(x, y - cell_height_ndc, 0.0, u_min, v_max);
    let p6 = Point::new(x + cell_width_ndc, y - cell_height_ndc, 0.0, u_max, v_max);

    [p1, p2, p3, p4, p5, p6]
}

impl Renderer {
    pub fn new(mode: RenderMode, device: Arc<wgpu::Device>, surface_format: wgpu::TextureFormat, config: &SurfaceConfiguration, size: PhysicalSize<u32>) -> Self {
        // initalize vertex data 
        let glyph_cache = GlyphCache::new(512, 512); 

        tracing::error!("size: width: {}, height: {}, config: width {} height: {}", size.width, size.height, config.width, config.height);
        let empty_instance: Vec<CellInstance> = Vec::new();

        let points: [Point; 6] = create_quad_for_char(-1.0, 1.0, 16.0, 16.0, 0.0, 0.0, size.width, size.height); 
        
        let byte_slice: &[u8] = bytemuck::cast_slice(&points);
        
        let vertex_buffer = BufferInitDescriptor {
            label: Some("LABEL"),
            contents: byte_slice,
            usage: BufferUsages::VERTEX,
        };

        let instance_buffer = BufferInitDescriptor {
            label: Some("Cell buffer"),
            contents: bytemuck::cast_slice(&empty_instance),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        };

        let max_cells = (size.width as f32 / CELL_WIDTH_PX) * (size.height as f32 / CELL_HEIGHT_PX);
        
        let buffer: wgpu::Buffer = device.create_buffer_init(&vertex_buffer);

        let instance_final_buffer: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cell buffer"),
            size: (max_cells as usize * size_of::<CellInstance>()) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });


        let glyph_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Glyph Texture"),
            size: wgpu::Extent3d {
                width: glyph_cache.atlas_width,
                height: glyph_cache.atlas_height,
                depth_or_array_layers: 1,
            },
            usage: TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            format: wgpu::TextureFormat::R8Unorm,
            dimension: TextureDimension::D2,
            view_formats: &[],
            sample_count: 1,
            mip_level_count: 1,
        });
        
        let glyph_texture_view = glyph_texture.create_view(&TextureViewDescriptor::default());

        let glyph_cache_sampler: wgpu::Sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });



        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("glyph cache bind group layout"),
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
        });

        let glyph_bind_group: BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Glyph Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&glyph_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&glyph_cache_sampler),
                },
            ],

        });




        let mut ret = Self {
            mode,
            device,
            surface_format,
            points,
            vertex_buffer: buffer,
            instance_buffer: instance_final_buffer,
            render_pipeline: None, 
            bind_group_layout: Some(bind_group_layout),
            bitmap_renderer: None,
            cell_instance_count: 0,
            glyph_cache,
            glyph_texture,
            size,
            glyph_bind_group,
        };

    
        let render_pipeline = ret.create_render_pipeline(&ret.device, config);

        if mode == RenderMode::Bitmap {
            ret.bitmap_renderer = Some(BitMapRenderer::new(Arc::clone(&ret.device), None, None, None));

        }else if mode == RenderMode::SDF {

        }else if mode == RenderMode::MSDF {

        }else {
            tracing::error!("Failed to Initialize Render - No Notable Renderer Selected") 
        }
            
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

        let quad_attributes: &[VertexAttribute] = &[
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

        let instance_vertex_attributes: &[VertexAttribute] = &[
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0, 
                shader_location: 2
            },
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 8, 
                shader_location: 3
            },
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 16, 
                shader_location: 4
            },
        ]; 


        let quad_layout: VertexBufferLayout = VertexBufferLayout {
            array_stride: size_of::<Point>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: quad_attributes,
        };


        let instance_buffer_layout: VertexBufferLayout = VertexBufferLayout {
            array_stride: size_of::<CellInstance>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: instance_vertex_attributes,
        };

        let layouts: &[VertexBufferLayout] = &[quad_layout, instance_buffer_layout];

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex_entry_point"),
                buffers: layouts, 
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


    pub fn draw(&mut self, render_pass: &mut wgpu::RenderPass, queue: &wgpu::Queue) {
        match self.mode {
            RenderMode::Bitmap => {
                // let bind_group = self.bitmap_renderer.as_mut().unwrap().render(render_pass, g, queue, self.bind_group_layout.as_ref().unwrap());
                // tracing::debug!("pasing glyph: {:?}", glyph);
                render_pass.set_bind_group(0, &self.glyph_bind_group, &[]);
                // render_pass.set_bind_group(0, bind_group.as_ref().unwrap(), &[]);
                render_pass.set_pipeline(self.render_pipeline.as_ref().unwrap());
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                render_pass.draw(0..6, 0..self.cell_instance_count);

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

    pub fn resolve_glyphs(&mut self, terminal: &Terminal, engine: &mut TextEngine, queue: &wgpu::Queue) -> Vec<CellInstance> {
        let mut finalized_grid: Vec<CellInstance> = Vec::new();
        let grid_w = terminal.cols;

        for index in 0..terminal.grid.len() {
            let cell: &Cell = terminal.grid.get(index).unwrap();

            if cell.state == CellState::NotWritten { continue; }

            let codepoint = cell.codepoint;
            let rasterized_glyph: RasterizedGlyph = engine.get_rasterized(codepoint);
            let is_new = self.glyph_cache.get(codepoint).is_none();
            let cached_glyph: Option<AtlasRect> = self.glyph_cache.get_or_allocate(codepoint, rasterized_glyph.width, rasterized_glyph.height);
            let rect = cached_glyph.unwrap();

            if is_new {
                // Deep regret not making this a func in Glyph ngl
                let size = wgpu::Extent3d {
                    width: rasterized_glyph.width,
                    height: rasterized_glyph.height,
                    depth_or_array_layers: 1
               };
                
                let data_layout = wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(rasterized_glyph.width), 
                    rows_per_image: Some(rasterized_glyph.height), 
                };

                let glyph_atlas_texel = &wgpu::TexelCopyTextureInfoBase {
                    texture: &self.glyph_texture,
                    mip_level: 0,
                    origin: Origin3d {
                        x: rect.x,  
                        y: rect.y, 
                        z: 0,
                    },
                    aspect: TextureAspect::All,    
                };

                queue.write_texture(*glyph_atlas_texel, &rasterized_glyph.data, data_layout, size);
    
            }

            if cached_glyph.is_none() {
                // full cache
                panic!("TODO: not do this");
            }

            let row = (index / grid_w as usize) as f32;
            let col = (index % grid_w as usize) as f32;

            let cell_width_ndc = (CELL_WIDTH_PX * 2.0) / self.size.width as f32;
            let cell_height_ndc = (CELL_HEIGHT_PX * 2.0) / self.size.height as f32; 

            let x_ndc = col * cell_width_ndc;
            let y_ndc = -(row * cell_height_ndc);


            let u_min = rect.x as f32 / self.glyph_texture.width() as f32;
            let u_max = (rect.x as f32 + rasterized_glyph.width as f32) / self.glyph_texture.width() as f32;

            let v_min = rect.y as f32 / self.glyph_texture.height() as f32;
            let v_max = (rect. y as f32 + rasterized_glyph.height as f32) / self.glyph_texture.height() as f32;
            
            let instance = CellInstance {
                x: x_ndc,
                y: y_ndc,

                u_min,
                u_max, 
                v_min,
                v_max,
            };

            finalized_grid.push(instance);
        }

        finalized_grid
    }


}
