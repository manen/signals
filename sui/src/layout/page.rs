use super::Layable;
use crate::{
	comp::{Comp, Compatible},
	Details,
};
use raylib::prelude::RaylibDrawHandle;

/// simple page layout, one element after another
#[derive(Clone, Debug, Default)]
pub struct Page<'a> {
	components: Vec<Comp<'a>>,
	horizontal: bool,
}
impl<'a> Page<'a> {
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

	pub fn render(&self, d: &mut RaylibDrawHandle, mut x: i32, mut y: i32, scale: f32) {
		for e in self.components.iter() {
			let (rw, rh) = e.d().size();
			e.d().render(
				d,
				Details {
					x,
					y,
					aw: rw,
					ah: rh,
				},
				scale,
			);
			if !self.horizontal {
				y += (rh as f32 * scale).floor() as i32;
			} else {
				x += (rw as f32 * scale).floor() as i32;
			}
		}
	}
}
impl<'a> Layable for Page<'a> {
	fn size(&self) -> (i32, i32) {
		self.components.iter().fold((0, 0), |a, layable| {
			let size = layable.d().size();
			if !self.horizontal {
				(a.0 + size.0, a.1.max(size.1))
			} else {
				(a.0.max(size.0), a.1 + size.1)
			}
		})
	}
	/// this implementation doesn't care about available width and height
	fn render(&self, d: &mut RaylibDrawHandle, det: Details, scale: f32) {
		Page::render(&self, d, det.x, det.y, scale);
	}

	fn pass_event(&self, event: super::Event) -> Option<super::Event> {
		match event {
			super::Event::MouseEvent { x: ptr_x, y: ptr_y } => {
				let (mut x, mut y) = (0, 0);
				for c in self.components.iter() {
					let (cw, ch) = c.size();
					if ptr_x >= x && ptr_x <= x + cw // x
					 && ptr_y >= y && ptr_y <= y + ch
					{
						return c.pass_event(super::Event::MouseEvent {
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
