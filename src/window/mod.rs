pub mod renderer;
mod device;
pub mod rasterizer;
mod text_engine;
mod glyph_cache;

use std::str::Chars;
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::Key::{Character, Named};
use winit::keyboard::NamedKey;
use winit::window::{Window, WindowId};

use crate::terminal::Terminal;
use crate::ttf::parser::TTFParser;
use crate::window::device::GPUState;
use crate::UserOptions;
use crate::window::text_engine::{RasterizedGlyph, TextEngine};


#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Point {
    x: f32,
    y: f32,
    z: f32,

    u: f32, 
    v: f32,
}

impl Point {
    fn new(x: f32, y: f32, z: f32, u: f32, v: f32) -> Self {
        Self {
            x, y, z, u, v
        }
    }
}


pub struct App {
    window: Option<Arc<Window>>,
    gpu: Option<GPUState>,

    engine: TextEngine,
    terminal: Terminal,
    // user customization
    user_options: UserOptions, 

}

impl App {
    pub fn new(options: &UserOptions) -> Self {
        let mut user_options = options.clone(); 
        let terminal: Terminal = Terminal::new(user_options.size.width, user_options.size.height);
        if let Ok(ttf_parser) = TTFParser::new(&user_options.font) {
            user_options.font.font_metric = Some(ttf_parser.font_metric);
            let engine = TextEngine::new(ttf_parser, user_options.clone());
            Self {
                window: None,
                gpu: None,
                user_options,
                engine,
                terminal
            }
        }else {
            tracing::error!("Failed To Initialize TTF Parser With Error: Defaulting to Default Settings");
            let ttf_default = TTFParser::new(&UserOptions::default().font).unwrap_or_else(|e| { panic!("Error: {}", e)} );

            user_options.font.font_metric = Some(ttf_default.font_metric);
            let engine = TextEngine::new(ttf_default, user_options.clone());

            Self {
                window: None,
                gpu: None,
                user_options,
                engine,
                terminal
            }

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
                    &self.user_options.renderer_mode, 
                    self.user_options.background_color,
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
                self.window.as_ref().unwrap().request_redraw();
                self.gpu.as_mut().unwrap().redraw_request(&self.terminal, &mut self.engine);
            },
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                if event.state != ElementState::Pressed { return }
                match event.logical_key {
                    Character(string) => {
                        if let Some(character) = string.chars().next() {
                            let codepoint = character as u32;
                            self.terminal.write_char(codepoint);
                        }
                    },
                    Named(NamedKey::Space) => { 
                        tracing::error!("Named(Space) branch");
                        self.terminal.write_char(' ' as u32); 
                    },
                    _ => {},


                }
                
                self.window.as_ref().unwrap().request_redraw();
                self.gpu.as_mut().unwrap().redraw_request(&self.terminal, &mut self.engine);
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

