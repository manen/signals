use super::Layable;
use crate::{
	comp::{Comp, Compatible},
	Details,
};
use raylib::prelude::RaylibDrawHandle;

/// simple page layout, one element after another
#[derive(Clone, Debug, Default)]
pub struct Page<'a> {
	elements: Vec<Comp<'a>>,
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
	pub fn new<I: Into<Vec<Comp<'a>>>>(elements: I, horizontal: bool) -> Self {
		Self {
			elements: elements.into(),
			horizontal,
		}
	}

	pub fn push<C: Compatible<'a>>(&mut self, c: impl Into<C>) {
		self.elements.push(c.into().into_comp());
	}

	pub fn render(&self, d: &mut RaylibDrawHandle, mut x: i32, mut y: i32, scale: f32) {
		for e in self.elements.iter() {
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
		self.elements.iter().fold((0, 0), |a, layable| {
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
}
