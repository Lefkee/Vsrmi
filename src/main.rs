mod app;
mod renderer;

use anyhow::Result;

fn main() -> Result<()> {
    renderer::install_panic_hook();

    let mut tui = renderer::Tui::new()?;
    let mut editor = app::App::new();
    let result = app::run(&mut editor, &mut tui);

    drop(tui);
    result
}
