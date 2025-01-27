pub mod core;
pub use core::{Details, DynamicLayable, Layable};

pub mod comp;
pub use comp::{Comp, Compatible, Div, SelectBar, Text};

pub mod dialog;

pub mod tex;

pub mod ui;
pub use ui::*;

pub const fn color(r: u8, g: u8, b: u8, a: u8) -> raylib::color::Color {
	raylib::color::Color { r, g, b, a }
}
