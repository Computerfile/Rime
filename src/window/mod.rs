pub mod renderer;
mod device;
pub mod rasterizer;
mod text_engine;
mod glyph_cache;

use std::str::Chars;
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalSize};
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
    cell_width_px: u16,
    cell_height_px: u16,

}

impl App {
    pub fn new(options: &UserOptions) -> Self {

        let mut user_options = options.clone(); 
        
        let ttf_parser = TTFParser::new(&options.font).unwrap_or_else(|e| { panic!("Error: {}", e)} );
        
        user_options.font.font_metric = Some(ttf_parser.font_metric);
        let mut engine = TextEngine::new(ttf_parser, user_options.clone());

        let advance_width = engine.get_advaned_widths('M' as u32).unwrap_or(0) as f32;
        let units_per_em = user_options.font.font_metric.unwrap().units_per_em as f32;
        let ascent = user_options.font.font_metric.unwrap().layout_info.ascent as f32;
        let descent = user_options.font.font_metric.unwrap().layout_info.descent as f32;

        let cell_width_px = ((advance_width / units_per_em) * user_options.font.font_size) as u16;
        let cell_height_px = (((ascent - descent) / units_per_em) * user_options.font.font_size * user_options.font.line_height) as u16;
        
        let terminal: Terminal = Terminal::new(user_options.size.width, user_options.size.height, cell_width_px, cell_height_px);

        Self {
            window: None,
            gpu: None,
            user_options,
            engine,
            terminal,
            cell_width_px,
            cell_height_px
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
                    self.user_options.font.font_size, 
                    &self.user_options.renderer_mode, 
                    self.user_options.background_color,
                    self.user_options.font.clone(),
                    self.cell_width_px,
                    self.cell_height_px
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
                    Named(NamedKey::Backspace) => { 
                        tracing::error!("Named(Space) branch");
                        self.terminal.delete_char(); 
                    },
                    _ => {},


                }
                
                self.window.as_ref().unwrap().request_redraw();
                self.gpu.as_mut().unwrap().redraw_request(&self.terminal, &mut self.engine);
            },
            WindowEvent::Resized(size) => {
                self.terminal.resize(size.width, size.height);
                self.gpu.as_mut().unwrap().resize(size);
                self.window.as_ref().unwrap().request_redraw();
                self.gpu.as_mut().unwrap().redraw_request(&self.terminal, &mut self.engine);
                
            },
            WindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer } => {},
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

