use crate::core::Layable;
use crate::{
	comp::{Comp, Compatible},
	Details,
};
use raylib::prelude::RaylibDrawHandle;

/// simple page layout, one element after another \
/// just imagine an html div
#[derive(Clone, Debug, Default)]
pub struct Box<'a> {
	components: Vec<Comp<'a>>,
	horizontal: bool,
}
impl<'a> Box<'a> {
	pub fn empty() -> Self {
		Self::default()
	}
	pub fn horizontal() -> Self {
		Self {
			horizontal: true,
			..Default::default()
		}
	}
	pub fn new<I: Into<Vec<Comp<'a>>>>(components: I, horizontal: bool) -> Self {
		Self {
			components: components.into(),
			horizontal,
		}
	}

	pub fn push<C: Compatible<'a>>(&mut self, c: C) {
		self.components.push(c.into_comp());
	}
}
impl<'a> Layable for Box<'a> {
	fn size(&self) -> (i32, i32) {
		self.components.iter().fold((0, 0), |a, layable| {
			let size = layable.size();
			if !self.horizontal {
				(a.0 + size.0, a.1.max(size.1))
			} else {
				(a.0.max(size.0), a.1 + size.1)
			}
		})
	}

	fn render(&self, d: &mut RaylibDrawHandle, det: Details, scale: f32) {
		let (box_w, box_h) = self.size();

		let (mut x, mut y) = (0, 0);
		for child in self.components.iter() {
			let (child_w, child_h) = child.size();

			let child_det = Details {
				x: det.x + x,
				y: det.y + y,
				aw: if !self.horizontal { box_h } else { child_w }, // I HAVE NO IDEA WHY THIS WORKS
				ah: if self.horizontal { box_w } else { child_h }, // why do box_w and box_h have to be swapped? idk
			};

			child.render(d, child_det, scale);
			if !self.horizontal {
				y += (child_det.ah as f32 * scale).floor() as i32;
			} else {
				x += (child_det.aw as f32 * scale).floor() as i32;
			}
		}
	}

	fn pass_event(&self, event: crate::core::Event) -> Option<crate::core::Event> {
		match event {
			crate::core::Event::MouseEvent { x: ptr_x, y: ptr_y } => {
				let (mut x, mut y) = (0, 0);
				for c in self.components.iter() {
					let (cw, ch) = c.size();
					if ptr_x >= x && ptr_x <= x + cw // x
					 && ptr_y >= y && ptr_y <= y + ch
					{
						return c.pass_event(crate::core::Event::MouseEvent {
							x: ptr_x - x,
							y: ptr_y - y,
						});
					} else {
						if !self.horizontal {
							y += ch;
						} else {
							x += cw;
						}
					}
				}
				None // mouseevent didn't intersect any of the components
			}
			_ => None,
		}
	}
}
