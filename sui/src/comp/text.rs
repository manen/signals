use std::borrow::Cow;

use raylib::{
	color::Color,
	math::Vector2,
	prelude::{RaylibDraw, RaylibDrawHandle},
};

use crate::Layable;

pub const BOUNDS_DEBUG: bool = false;
pub const SPACING: f32 = 1.0; //idk idc
pub const DEFAULT_COLOR: Color = Color::WHITE;

/// Font is currently a placeholder for if fonts were ever to be implemented,
/// currently font has one variant and it'll render using the default font
pub struct Font;

pub struct Text<'a>(Cow<'a, str>, i32, Font, Color);

impl<'a> Text<'a> {
	pub fn new<I: Into<Cow<'a, str>>>(text: I, size: i32) -> Self {
		Self(text.into(), size, Font, DEFAULT_COLOR)
	}
}
impl<'a, I: Into<Cow<'a, str>>> Into<Text<'a>> for (I, i32) {
	fn into(self) -> Text<'a> {
		Text::new(self.0, self.1)
	}
}

impl<'a> Layable for Text<'a> {
	fn size(&self) -> (i32, i32) {
		let font = unsafe { raylib::ffi::GetFontDefault() };

		let cstring = std::ffi::CString::new(self.0.as_ref())
			.expect("CString::new failed while measuring text size:(");

		let dimensions =
			unsafe { raylib::ffi::MeasureTextEx(font, cstring.as_ptr(), self.1 as f32, SPACING) };

		(dimensions.x.ceil() as i32, dimensions.y.ceil() as i32)
	}
	fn render(&self, d: &mut RaylibDrawHandle, det: crate::Details, scale: i32) {
		if BOUNDS_DEBUG {
			let s = self.size();
			d.draw_rectangle_lines(det.x, det.y, s.0, s.1, Color::WHITE);
		}

		d.draw_text_ex(
			d.get_font_default(),
			&self.0,
			Vector2::new(det.x as f32, det.y as f32),
			self.1 as f32,
			SPACING,
			self.3,
		);
	}
}
