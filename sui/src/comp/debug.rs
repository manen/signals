use raylib::prelude::RaylibDraw;

use crate::Layable;

#[derive(Clone, Debug)]
/// `Debug` renders some useful ui debug info: \
///
/// - layable.size() in red
/// - self.render det in blue
pub struct Debug<L: Layable> {
	layable: L,
}
impl<L: Layable> Debug<L> {
	pub fn new(layable: L) -> Self {
		Self { layable }
	}
}
impl<L: Layable> Layable for Debug<L> {
	fn size(&self) -> (i32, i32) {
		self.layable.size()
	}
	fn render(&self, d: &mut raylib::prelude::RaylibDrawHandle, det: crate::Details, scale: f32) {
		use raylib::color::Color;

		let size = self.layable.size();
		d.draw_rectangle_lines(
			det.x,
			det.y,
			(size.0 as f32 * scale) as _,
			(size.1 as f32 * scale) as _,
			Color::RED,
		);

		d.draw_rectangle_lines(
			det.x,
			det.y,
			(det.aw as f32 * scale) as _,
			(det.ah as f32 * scale) as _,
			Color::BLUE,
		);

		self.layable.render(d, det, scale)
	}
	fn pass_event(
		&self,
		event: crate::core::Event,
		det: crate::Details,
		scale: f32,
	) -> Option<crate::core::Event> {
		self.layable.pass_event(event, det, scale)
	}
}

/// See [Debug]
pub trait Debuggable: Sized + Layable {
	fn debug(self) -> Debug<Self> {
		Debug::new(self)
	}
}
impl<L: Layable> Debuggable for L {}
