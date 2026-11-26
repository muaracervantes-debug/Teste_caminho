use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::renderer::Renderer;

pub struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            renderer: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = Window::default_attributes()
            .with_title("Teste Caminho - Mundo 3D")
            .with_inner_size(PhysicalSize::new(1280, 720));

        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                eprintln!("Falha ao criar a janela: {error}");
                event_loop.exit();
                return;
            }
        };

        match pollster::block_on(Renderer::new(window.clone())) {
            Ok(renderer) => {
                self.window = Some(window.clone());
                self.renderer = Some(renderer);
                window.request_redraw();
            }
            Err(error) => {
                eprintln!("Falha ao inicializar a renderizacao: {error}");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref() else {
            return;
        };

        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size);
                }
                window.request_redraw();
            }

            WindowEvent::RedrawRequested => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.render();
                }
                window.request_redraw();
            }

            WindowEvent::Occluded(false) => window.request_redraw(),

            _ => {}
        }
    }
}
