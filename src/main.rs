mod app;
mod camera;
mod diagnostics;
mod renderer;

use std::error::Error;
use winit::event_loop::EventLoop;

fn main() {
    diagnostics::install_panic_hook();

    if let Err(error) = run() {
        diagnostics::record(&format!("Falha ao executar a aplicacao: {error}"));
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = app::App::default();

    event_loop.run_app(&mut app)?;

    Ok(())
}
