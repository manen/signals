use crate::{
	core::{Event, MouseEvent},
	Layable,
};

#[derive(Clone)]
/// while this technically does work with any Layable, to implement Compatible C needs to be Comp
pub struct Clickable<C, F, T>
where
	T: Clone + 'static,
	F: Fn((i32, i32)) -> T,
	C: Layable,
{
	comp: C,
	gen_ret: F,
	/// if true, it will check if self.comp bubbles anything back and only respond if it doesn't
	fallback: bool,
}
impl<C: Layable + std::fmt::Debug, T: Clone, F: Fn((i32, i32)) -> T> std::fmt::Debug
	for Clickable<C, F, T>
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Clickable")
			.field("comp", &self.comp)
			.field("fallback", &self.fallback)
			.finish()
	}
}
impl<C: Layable, T: Clone, F: Fn((i32, i32)) -> T> Clickable<C, F, T> {
	pub fn new(gen_ret: F, comp: C) -> Self {
		Clickable {
			comp,
			gen_ret,
			fallback: false,
		}
	}
	pub fn new_fallback(gen_ret: F, comp: C) -> Self {
		Clickable {
			comp,
			gen_ret,
			fallback: true,
		}
	}

	pub fn take(self) -> C {
		self.comp
	}
}
impl<T: Clone, C: Layable, F: Fn((i32, i32)) -> T> Layable for Clickable<C, F, T> {
	fn size(&self) -> (i32, i32) {
		self.comp.size()
	}

	fn render(&self, d: &mut crate::Handle, det: crate::Details, scale: f32) {
		self.comp.render(d, det, scale)
	}

	fn pass_event(
		&self,
		event: Event,
		det: crate::Details,
		scale: f32,
	) -> Option<crate::core::ReturnEvent> {
		let respond = || match event {
			Event::MouseEvent(MouseEvent::MouseClick { x, y }) => {
				if det.is_inside(x, y) {
					Some(Event::ret((self.gen_ret)((x, y))))
				} else {
					None
				}
			}
			_ => None,
		};

		if !self.fallback {
			match respond() {
				Some(a) => Some(a),
				None => self.comp.pass_event(event, det, scale),
			}
		} else {
			if let Some(comp_resp) = self.comp.pass_event(event, det, scale) {
				Some(comp_resp)
			} else {
				respond()
			}
		}
	}
}
