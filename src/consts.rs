use raylib::color::Color;
use raylib::ffi::{KeyboardKey, MouseButton};

const fn color(r: u8, g: u8, b: u8, a: u8) -> Color {
	Color { r, g, b, a }
}

pub const BACKGROUND: Color = Color::BLACK;
pub const WIRE_ON: Color = color(230, 200, 200, 255);
pub const WIRE_OFF: Color = color(80, 80, 80, 255);
pub const SWITCH_ON: Color = color(200, 200, 200, 255);
pub const SWITCH_OFF: Color = color(100, 100, 100, 255);
pub const NOT_BASE: Color = color(39, 143, 86, 255);
pub const NOT_ON: Color = color(82, 81, 80, 255);
pub const NOT_OFF: Color = color(255, 255, 255, 255);

pub const TOOL_SWITCH: KeyboardKey = KeyboardKey::KEY_T;
pub const TOOL_USE: MouseButton = MouseButton::MOUSE_BUTTON_LEFT;
