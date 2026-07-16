mod app;
mod config;
mod editor;
mod filesystem;
mod renderer;
mod theme;

use anyhow::Result;

fn main() -> Result<()> {
    renderer::install_panic_hook();

    let mut tui = renderer::Tui::new()?;
    let mut editor = app::App::new();
    let result = app::run(&mut editor, &mut tui);

    drop(tui);
    result
}
