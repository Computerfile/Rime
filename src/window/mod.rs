mod rendering;

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use crate::window::rendering::GPUState;
use crate::UserOptions;



pub struct App {
    window: Option<Arc<Window>>,
    gpu: Option<GPUState>,

    // user customization
    user_options: UserOptions, 
}

impl App {
    pub fn new(user_options: UserOptions) -> Self {
        Self {
            window: None,
            gpu: None,
            user_options,
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
        self.gpu = Some(pollster::block_on(GPUState::new(window, 
                    self.user_options.font_size, 
                    self.user_options.line_height)).unwrap());
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
            /*WindowEvent::Resized => {},
            WindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer } => {},*/
            _ => (),
        }
    }
}

pub fn init_app(user_options: UserOptions) {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new(user_options);

    event_loop.run_app(&mut app);
}

