use crate::{core::Event, Details, Layable};

#[derive(Clone, Debug)]
/// self.size() is self.layable.size(), the centering only happens on self.render()
pub struct Centered<L: Layable> {
	layable: L,
}
impl<L: Layable> Centered<L> {
	pub fn new(layable: L) -> Self {
		Self { layable }
	}

	fn l_det(&self, det: Details, scale: f32) -> Details {
		let (l_w, l_h) = self.layable.size();

		let (base_x, base_y) = (
			det.x + (det.aw as f32 / 2.0 - l_w as f32 / 2.0 * scale) as i32,
			det.y + (det.ah as f32 / 2.0 - l_h as f32 / 2.0 * scale) as i32,
		);
		let l_det = crate::Details {
			x: base_x,
			y: base_y,
			aw: det.aw - base_x,
			ah: det.ah - base_y,
		};

		l_det
	}
}
impl<L: Layable> Layable for Centered<L> {
	fn size(&self) -> (i32, i32) {
		self.layable.size()
	}
	fn render(&self, d: &mut crate::Handle, det: crate::Details, scale: f32) {
		self.layable.render(d, self.l_det(det, scale), scale);
	}
	fn pass_event(
		&self,
		event: Event,
		det: crate::Details,
		scale: f32,
	) -> Option<crate::core::ReturnEvent> {
		self.layable
			.pass_event(event, self.l_det(det, scale), scale)
	}
}
