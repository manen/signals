use crate::{
	comp::{Comp, Compatible},
	Details,
};
use raylib::prelude::RaylibDrawHandle;

pub trait Layable {
	fn size(&self) -> (i32, i32);

	fn render(&self, d: &mut RaylibDrawHandle, det: Details, scale: i32);
}

/// simple page layout, one element after another
#[derive(Clone, Debug, Default)]
pub struct Page<'a> {
	elements: Vec<Comp<'a>>,
}
impl<'a> Page<'a> {
	pub fn empty() -> Self {
		Self::default()
	}
	pub fn new<I: Into<Vec<Comp<'a>>>>(elements: I) -> Self {
		Self {
			elements: elements.into(),
		}
	}

	pub fn push<C: Compatible<'a>>(&mut self, c: impl Into<C>) {
		self.elements.push(c.into().into_comp());
	}

	pub fn render(&self, d: &mut RaylibDrawHandle, x: i32, mut y: i32, scale: i32) {
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
			y += rh;
		}
	}
}
impl<'a> Layable for Page<'a> {
	fn size(&self) -> (i32, i32) {
		self.elements.iter().fold((0, 0), |a, layable| {
			let size = layable.d().size();
			(a.0 + size.0, a.1.max(size.1))
		})
	}
	/// this implementation doesn't care about available width and height
	fn render(&self, d: &mut RaylibDrawHandle, det: Details, scale: i32) {
		Page::render(&self, d, det.x, det.y, scale);
	}
}

// make a text component that returns a correct size
// from there make a button or something
