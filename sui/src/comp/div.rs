use crate::core::Layable;
use crate::{
	comp::{Comp, Compatible},
	Details,
};
use raylib::prelude::{RaylibDraw, RaylibDrawHandle};

/// simple page layout, one element after another \
/// just imagine an html div
#[derive(Clone, Debug, Default)]
pub struct Div<'a> {
	components: Vec<Comp<'a>>,
	horizontal: bool,
}
impl<'a> Div<'a> {
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
impl<'a> Layable for Div<'a> {
	fn size(&self) -> (i32, i32) {
		let (mut w, mut h) = (0, 0);

		for comp in self.components.iter() {
			let (comp_w, comp_h) = comp.size();

			if !self.horizontal {
				(w, h) = (w.max(comp_w), h + comp_h)
			} else {
				(w, h) = (w + comp_w, h.max(comp_h))
			}
		}

		(w, h)
	}

	fn render(&self, d: &mut RaylibDrawHandle, det: Details, scale: f32) {
		let (self_w, self_h) = self.size();

		let (mut x, mut y) = (det.x, det.y);
		for comp in self.components.iter() {
			let (comp_w, comp_h) = comp.size();
			let comp_det = Details {
				x,
				y,
				aw: if !self.horizontal {
					(self_w as f32 * scale) as i32
				} else {
					comp_w
				},
				ah: if self.horizontal {
					(self_h as f32 * scale) as i32
				} else {
					comp_h
				},
			};

			comp.render(d, comp_det, scale);

			if !self.horizontal {
				y += (comp_h as f32 * scale) as i32;
			} else {
				x += (comp_w as f32 * scale) as i32;
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
