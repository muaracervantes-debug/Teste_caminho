use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::{diagnostics, renderer::Renderer};

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
                diagnostics::record(&format!("Falha ao criar a janela: {error}"));
                event_loop.exit();
                return;
            }
        };

        self.window = Some(window.clone());
        diagnostics::record("Janela criada. Iniciando renderer.");

        // Captura falhas/panics do driver ou da inicializacao do wgpu para
        // registrar a causa em arquivo, em vez de encerrar sem diagnostico.
        match catch_unwind(AssertUnwindSafe(|| {
            pollster::block_on(Renderer::new(window.clone()))
        })) {
            Ok(Ok(renderer)) => {
                diagnostics::record("Renderer inicializado com sucesso.");
                self.renderer = Some(renderer);
                window.request_redraw();
            }
            Ok(Err(error)) => {
                diagnostics::record(&format!("Falha ao inicializar a renderizacao: {error}"));
                window.set_title("Falha grafica - veja teste_caminho.log");
            }
            Err(payload) => {
                let message = diagnostics::panic_message(payload.as_ref());
                diagnostics::record(&format!(
                    "PANIC capturado durante a inicializacao do renderer: {message}"
                ));
                window.set_title("Panic grafico - veja teste_caminho.log");
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
            WindowEvent::CloseRequested => {
                diagnostics::record("Evento CloseRequested recebido pela janela.");
                event_loop.exit();
            },

            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size);
                }
                window.request_redraw();
            }

            WindowEvent::RedrawRequested => {
                let render_result = self.renderer.as_mut().map(|renderer| {
                    catch_unwind(AssertUnwindSafe(|| renderer.render()))
                });

                if let Some(Err(payload)) = render_result {
                    let message = diagnostics::panic_message(payload.as_ref());
                    diagnostics::record(&format!(
                        "PANIC capturado durante a renderizacao: {message}"
                    ));
                    self.renderer = None;
                    window.set_title("Panic durante renderizacao - veja teste_caminho.log");
                }

                if self.renderer.is_some() {
                    window.request_redraw();
                }
            }

            WindowEvent::Occluded(false) => window.request_redraw(),

            _ => {}
        }
    }
}
