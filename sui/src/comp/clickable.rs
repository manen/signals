use crate::{core::Event, Layable};

#[derive(Clone, Debug)]
/// while this technically does work with any Layable, to implement Compatible C needs to be Comp
pub struct Clickable<C, T>
where
	T: Clone + 'static,
	C: Layable,
{
	comp: C,
	ret: T,
	/// if true, it will check if self.comp bubbles anything back and only respond if it doesn't
	fallback: bool,
}
impl<C: Layable, T: Clone> Clickable<C, T> {
	pub fn new(ret: T, comp: C) -> Self {
		Clickable {
			comp,
			ret,
			fallback: false,
		}
	}
	pub fn new_fallback(ret: T, comp: C) -> Self {
		Clickable {
			comp,
			ret,
			fallback: true,
		}
	}

	pub fn take(self) -> C {
		self.comp
	}
}
impl<T: Clone, C: Layable> Layable for Clickable<C, T> {
	fn size(&self) -> (i32, i32) {
		self.comp.size()
	}

	fn render(&self, d: &mut raylib::prelude::RaylibDrawHandle, det: crate::Details, scale: f32) {
		self.comp.render(d, det, scale)
	}

	fn pass_event(
		&self,
		event: Event,
		det: crate::Details,
		scale: f32,
	) -> Option<crate::core::ReturnEvent> {
		let respond = || match event {
			Event::MouseClick { x, y } if det.mul_size(scale).is_inside(x, y) => {
				Some(Event::ret(self.ret.clone()))
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
