pub mod renderer;
mod device;
pub mod rasterizer;
mod text_engine;

use std::str::Chars;
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use crate::ttf::parser::TTFParser;
use crate::window::device::GPUState;
use crate::UserOptions;
use crate::window::renderer::Renderer;
use crate::window::text_engine::{RasterizedGlyph, TextEngine};


#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Point {
    x: f32,
    y: f32,
    z: f32,
}


pub struct App {
    window: Option<Arc<Window>>,
    gpu: Option<GPUState>,

    engine: TextEngine,

    // user customization
    user_options: UserOptions, 

}

impl App {
    pub fn new(options: &UserOptions) -> Self {
        let mut user_options = options.clone(); 
        let ttf_parser = TTFParser::new(&user_options.font);
        user_options.font.font_metric = Some(ttf_parser.font_metric);
        let engine = TextEngine::new(ttf_parser, user_options.clone());
        Self {
            window: None,
            gpu: None,
            user_options,
            engine,
        }
    }

}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title(self.user_options.title.clone())
            .with_inner_size(self.user_options.size);
        self.window = Some(Arc::new(event_loop.create_window(attrs).unwrap()));
        let window = self.window.as_ref().unwrap().clone();
        self.gpu = Some(pollster::block_on(
                GPUState::new(window, 
                    self.user_options.font_size, 
                    self.user_options.line_height, 
                    &self.user_options.renderer_mode
                )).unwrap());
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                self.window.as_ref().unwrap().request_redraw();
                self.gpu.as_mut().unwrap().redraw_request();
            },
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                if event.state != ElementState::Pressed { return }
                if let winit::keyboard::Key::Character(string) = event.logical_key {
                    if let Some(character) = string.chars().next() {
                        let codepoint = character as u32;
                        let rasterized_glyph: RasterizedGlyph = self.engine.get_rasterized(codepoint);
                        self.gpu.as_mut().unwrap().update_pending_glyph(rasterized_glyph);
                        self.window.as_ref().unwrap().request_redraw();
                    }

                }
                
            }
            /*WindowEvent::Resized => {},
            WindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer } => {},*/
            _ => (),
        }
    }
}

pub fn init_app(user_options: &UserOptions) {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new(user_options);

    event_loop.run_app(&mut app);
}

