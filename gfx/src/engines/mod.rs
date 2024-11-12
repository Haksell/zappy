mod console;

pub use console::render;

pub trait Engine {
    fn render() -> Result<(), Box<dyn std::error::Error>>;
}
