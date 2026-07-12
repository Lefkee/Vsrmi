mod app;
mod renderer;

use anyhow::Result;

fn main() -> Result<()> {
    renderer::install_panic_hook();
    let _tui = renderer::Tui::new()?;
    Ok(())
}
