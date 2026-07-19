mod app;
mod clipboard;
mod config;
mod editor;
mod filesystem;
mod input;
mod renderer;
mod theme;
mod ui;
mod undo;

use anyhow::Result;

fn main() -> Result<()> {
    renderer::install_panic_hook();

    let mut tui = renderer::Tui::new()?;
    let mut editor = app::App::new();
    let result = app::run(&mut editor, &mut tui);

    drop(tui);
    result
}
