use std::borrow::Cow;

use raylib::{color::Color, math::Vector2, prelude::RaylibDraw};

use crate::Layable;

pub const BOUNDS_DEBUG: bool = false;
pub const SPACING: f32 = 1.0; // idk idc
pub const DEFAULT_COLOR: Color = Color::WHITE;
// pub const DEFAULT_COLOR: Color = crate::color(195, 36, 209, 255);

/// Font is currently a placeholder for if fonts were ever to be implemented,
/// currently font has one variant and it'll render using the default font
#[derive(Debug, Clone)]
pub struct Font;

#[derive(Debug, Clone)]
pub struct Text<'a>(pub Cow<'a, str>, pub i32, Font, Color);

impl<'a> Text<'a> {
	pub fn new<I: Into<Cow<'a, str>>>(text: I, size: i32) -> Self {
		Self(text.into(), size, Font, DEFAULT_COLOR)
	}

	pub fn new_colored<I: Into<Cow<'a, str>>>(text: I, size: i32, color: Color) -> Self {
		Self(text.into(), size, Font, color)
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

		let measure_line = |text| {
			let cstring = std::ffi::CString::new(text)
				.expect("CString::new failed while measuring text size:(");

			let dimensions = unsafe {
				raylib::ffi::MeasureTextEx(font, cstring.as_ptr(), self.1 as f32, SPACING)
			};

			(dimensions.x.ceil() as i32, dimensions.y.ceil() as i32)
		};

		self.0
			.split('\n')
			.map(measure_line)
			.fold((0, 0), |acc, (x, y)| (acc.0.max(x), acc.1 + y - 1)) // i don't know why we need to remove 1 pixel from height per line
	}
	fn render(&self, d: &mut crate::Handle, det: crate::Details, scale: f32) {
		if BOUNDS_DEBUG {
			let s = self.size();
			d.draw_rectangle_lines(det.x, det.y, s.0, s.1, Color::WHITE);
		}

		let font = d.get_font_default();
		d.draw_text_ex(
			font,
			&self.0,
			Vector2::new(det.x as f32, det.y as f32),
			self.1 as f32 * scale,
			SPACING,
			self.3,
		);
	}

	fn pass_event(
		&self,
		_: crate::core::Event,
		_: crate::Details,
		_: f32,
	) -> Option<crate::core::ReturnEvent> {
		None
	}
}
