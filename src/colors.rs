use raylib::color::Color;

const fn color(r: u8, g: u8, b: u8, a: u8) -> Color {
	Color { r, g, b, a }
}

pub const BACKGROUND: Color = Color::BLACK;
pub const WIRE_ON: Color = color(230, 200, 200, 255);
pub const WIRE_OFF: Color = color(80, 80, 80, 255);
