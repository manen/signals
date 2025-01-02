use crate::{core::Event, Layable};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// while this technically does work with any Layable, to implement Compatible C needs to be Comp
pub struct Clickable<C: Layable> {
	comp: C,
	id: &'static str,
	n: i32,
	/// if true, it will check if self.comp bubbles anything back and only respond if it doesn't
	fallback: bool,
}
impl<C: Layable> Clickable<C> {
	pub fn new(comp: C, id: &'static str, n: i32) -> Self {
		Clickable {
			comp,
			id,
			n,
			fallback: false,
		}
	}
	pub fn new_fallback(comp: C, id: &'static str, n: i32) -> Self {
		Clickable {
			comp,
			id,
			n,
			fallback: true,
		}
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

	fn pass_event(&self, event: Event, det: crate::Details, scale: f32) -> Option<Event> {
		let respond = || match event {
			Event::MouseEvent { x, y } if det.mul_size(scale).is_inside(x, y) => {
				Some(Event::Named {
					id: self.id,
					n: self.n,
				})
			}
			_ => None,
		};

		if !self.fallback {
			respond()
		} else {
			if let Some(comp_resp) = self.comp.pass_event(event, det, scale) {
				Some(comp_resp)
			} else {
				respond()
			}
		}
	}
}
