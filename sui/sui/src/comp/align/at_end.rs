use crate::{Details, Layable};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
	Vert,
	Horiz,
	Both,
}
impl Mode {
	pub fn vert(&self) -> bool {
		match self {
			Self::Vert => true,
			Self::Horiz => false,
			Self::Both => true,
		}
	}
	pub fn horiz(&self) -> bool {
		match self {
			Self::Vert => false,
			Self::Horiz => true,
			Self::Both => true,
		}
	}
}

#[derive(Clone, Debug)]
pub struct AtEnd<L: Layable> {
	layable: L,
	mode: Mode,
}
impl<L: Layable> AtEnd<L> {
	pub fn new(mode: Mode, layable: L) -> Self {
		Self { layable, mode }
	}

	pub fn to_right(layable: L) -> Self {
		Self::new(Mode::Vert, layable)
	}
	pub fn to_bottom(layable: L) -> Self {
		Self::new(Mode::Horiz, layable)
	}
	pub fn to_bottom_right(layable: L) -> Self {
		Self::new(Mode::Both, layable)
	}

	fn l_det(&self, det: Details, scale: f32) -> Details {
		let (lw, lh) = self.layable.size();

		Details {
			x: if self.mode.vert() {
				det.x + ((det.aw as f32 - lw as f32) * scale) as i32
			} else {
				det.x
			},
			y: if self.mode.horiz() {
				det.y + ((det.ah as f32 - lh as f32) * scale) as i32
			} else {
				det.y
			},
			aw: lw,
			ah: lh,
		}
	}
}
impl<L: Layable> Layable for AtEnd<L> {
	fn size(&self) -> (i32, i32) {
		self.layable.size()
	}
	fn render(&self, d: &mut crate::Handle, det: crate::Details, scale: f32) {
		self.layable.render(d, self.l_det(det, scale), scale);
	}
	fn pass_event(
		&self,
		event: crate::core::Event,
		det: Details,
		scale: f32,
	) -> Option<crate::core::ReturnEvent> {
		self.layable
			.pass_event(event, self.l_det(det, scale), scale)
	}
}
