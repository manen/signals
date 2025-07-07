use raylib::prelude::RaylibDraw;

use crate::Layable;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// Color is a simple primitive that just renders a single color on the given det \
/// useful for a plain background
pub struct Color {
	color: raylib::color::Color,
}
impl Color {
	pub fn new(color: raylib::color::Color) -> Self {
		Color { color }
	}
}
impl Layable for Color {
	fn size(&self) -> (i32, i32) {
		(0, 0)
	}
	fn render(&self, d: &mut crate::Handle, det: crate::Details, scale: f32) {
		d.draw_rectangle(
			det.x,
			det.y,
			(det.aw as f32 * scale) as i32,
			(det.ah as f32 * scale) as i32,
			self.color,
		);
	}
	fn pass_event(
		&self,
		_event: crate::core::Event,
		_det: crate::Details,
		_scale: f32,
	) -> Option<crate::core::ReturnEvent> {
		None
	}
}
