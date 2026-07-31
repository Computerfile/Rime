use std::iter::once;
use std::sync::Arc;
use wgpu::{CurrentSurfaceTexture};
use wgpu::hal::SurfaceError;
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::window::renderer::{RenderMode, Renderer};
use crate::window::text_engine::RasterizedGlyph;

// tick of the GPU rendering loop
pub struct GPUState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: Arc<wgpu::Device>,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,

    pending_glyph: Option<RasterizedGlyph>,

    // misc
    size: PhysicalSize<u32>,
    renderer: Renderer
}    

impl GPUState {

    pub fn update_pending_glyph(&mut self, glyph: RasterizedGlyph) {
        self.pending_glyph = Some(glyph); 
    } 
    

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }


    fn render(&mut self) -> Result<(), SurfaceError> {
        let output = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(t) => t,
            CurrentSurfaceTexture::Suboptimal(t) => t,
            CurrentSurfaceTexture::Timeout => return Err(SurfaceError::Timeout),
            CurrentSurfaceTexture::Occluded => return Err(SurfaceError::Occluded),
            CurrentSurfaceTexture::Lost => return Err(SurfaceError::Lost),
            CurrentSurfaceTexture::Validation => return Err(SurfaceError::Lost),
            CurrentSurfaceTexture::Outdated => return Err(SurfaceError::Outdated),
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor{ label: Some("Render Encoder") }
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 1.0, g: 0.2, b: 0.3, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            self.renderer.draw(&mut render_pass, self.pending_glyph.as_ref(), &self.queue);
            
        }

        self.queue.submit(once(encoder.finish()));
        output.present();
        Ok(())
    }


    pub async fn new(window: Arc<Window>, font_size: f32, line_height: f32, render_mode: &RenderMode) -> anyhow::Result<Self> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });
        

        let surface = instance.create_surface(window.clone())?; 

        let adapter = instance.enumerate_adapters(wgpu::Backends::all()).await.into_iter().filter(|adapter| {
            adapter.is_surface_supported(&surface)
        }).next().expect("Failed to create adapter");
        

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;
        
        let device = Arc::new(device);

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
          .find(|f| f.is_srgb())
          .copied()
          .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let renderer = Renderer::new(*render_mode, device.clone(), surface_format, &config); 

        Ok(Self {
          window,
          surface,
          device: device,
          queue,
          config,
          pending_glyph: None,
          is_surface_configured: true,
          size,
          renderer: renderer
        })
    }

    pub fn redraw_request(&mut self) {
        match self.render() {
            Ok(()) => {},
            Err(e) => {tracing::warn!("Render Error {:?}", e)},
        } 

    }
    
}
