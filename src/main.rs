mod app;

use app::mode::Mode;

fn main() {
    println!("termi — mode: {}", Mode::default().name());
}
