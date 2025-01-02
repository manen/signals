use crate::Layable;

#[derive(Copy, Clone, Debug)]
/// Space is literally just some empty space
pub struct Space {
	w: i32,
	h: i32,
}
impl Space {
	pub fn new(w: i32, h: i32) -> Self {
		Self { w, h }
	}
}
impl Layable for Space {
	fn size(&self) -> (i32, i32) {
		(self.w, self.h)
	}
	fn render(&self, _: &mut raylib::prelude::RaylibDrawHandle, _: crate::Details, _: f32) {}
	fn pass_event(&self, _: crate::core::Event) -> Option<crate::core::Event> {
		None
	}
}
