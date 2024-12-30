use crate::{core::Event, Layable};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// while this technically does work with any Layable, to implement Compatible C needs to be Comp
pub struct Clickable<C: Layable> {
	comp: C,
	id: &'static str,
	n: i32,
}
impl<C: Layable> Clickable<C> {
	pub fn new(comp: C, id: &'static str, n: i32) -> Self {
		Clickable { comp, id, n }
	}
	pub fn take(self) -> C {
		self.comp
	}
}
impl<C: Layable> Layable for Clickable<C> {
	fn size(&self) -> (i32, i32) {
		self.comp.size()
	}

	fn render(&self, d: &mut raylib::prelude::RaylibDrawHandle, det: crate::Details, scale: f32) {
		self.comp.render(d, det, scale)
	}

	fn pass_event(&self, event: Event) -> Option<Event> {
		match event {
			Event::MouseEvent { .. } => Some(Event::Named {
				id: self.id,
				n: self.n,
			}),
			_ => None,
		}
	}
}
