use crate::Details;
use raylib::prelude::RaylibDrawHandle;
use std::borrow::Cow;

pub trait Layable {
	fn size(&self) -> (i32, i32);

	fn render(&self, d: &mut RaylibDrawHandle, det: Details, scale: i32);
}

/// simple page layout, one element after another
pub struct Page<'a> {
	elements: Cow<'a, [&'a dyn Layable]>,
}
impl<'a> Page<'a> {
	pub fn new<I: Into<Cow<'a, [&'a dyn Layable]>>>(elements: I) -> Self {
		Self {
			elements: elements.into(),
		}
	}

	pub fn render(&self, d: &mut RaylibDrawHandle, x: i32, mut y: i32, scale: i32) {
		for e in self.elements.iter() {
			let (rw, rh) = e.size();
			e.render(
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

// make a text component that returns a correct size
// from there make a button or something
