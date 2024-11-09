use raylib::color::Color;
use raylib::ffi::{KeyboardKey, MouseButton};

pub const fn color(r: u8, g: u8, b: u8, a: u8) -> Color {
	Color { r, g, b, a }
}

pub const DEBUG_WIRES: bool = false;
pub const DEBUG_CHUNKS: bool = false;
pub const DEBUG_NOT: bool = false;

pub const SELECT_BAR_SELECTED: Color = color(240, 240, 240, 255);
pub const SELECT_BAR_UNSELECTED: Color = color(160, 160, 160, 255);

pub const TICK_TIME: f32 = 0.03;

pub const BACKGROUND: Color = Color::BLACK;
pub const WIRE_ON: Color = color(230, 200, 200, 255);
pub const WIRE_OFF: Color = color(80, 80, 80, 255);
pub const SWITCH_ON: Color = color(200, 200, 200, 255);
pub const SWITCH_OFF: Color = color(100, 100, 100, 255);
pub const NOT_BASE: Color = color(39, 143, 86, 255);
pub const NOT_ON: Color = color(82, 81, 80, 255);
pub const NOT_OFF: Color = color(255, 255, 255, 255);

pub const TOOL_SWITCH: KeyboardKey = KeyboardKey::KEY_T;
pub const MOVE_UP: KeyboardKey = KeyboardKey::KEY_W;
pub const MOVE_DOWN: KeyboardKey = KeyboardKey::KEY_S;
pub const MOVE_LEFT: KeyboardKey = KeyboardKey::KEY_A;
pub const MOVE_RIGHT: KeyboardKey = KeyboardKey::KEY_D;
pub const TOOL_USE: MouseButton = MouseButton::MOUSE_BUTTON_LEFT;

pub const MOVE_AMOUNT: f32 = 5000.0;
