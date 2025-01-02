use std::borrow::Cow;

use raylib::{ffi::MouseButton, prelude::RaylibDrawHandle, RaylibHandle};

use crate::{
	comp::{self, Comp, Compatible},
	core::Event,
	Details, Layable,
};

/// this will turn any layable into a Comp \
///! do not use if Comp has an enum variant for the layable you're using,
///  as it will create a DynamicLayable for nothing (causing performance & memory overhead)
pub fn custom<'a, L: Layable + std::fmt::Debug + Clone + 'a>(layable: L) -> Comp<'a> {
	crate::DynamicLayable::new(layable).into_comp()
}

pub fn page<'a>(components: impl Into<Vec<Comp<'a>>>) -> Comp<'a> {
	comp::Div::new(components, false).into_comp()
}
pub fn page_h<'a>(components: impl Into<Vec<Comp<'a>>>) -> Comp<'a> {
	comp::Div::new(components, true).into_comp()
}
pub fn text<'a, T: Into<Cow<'a, str>>>(text: T, size: i32) -> Comp<'a> {
	comp::Text::new(text, size).into_comp()
}

/// `RootContext` contains everything needed to calculate Details and scales, for both rendering
/// and events. this is so there's no way [Layable::render] and [Layable::pass_event]
/// could work with different data.
pub struct RootContext<'a, L: Layable> {
	layable: &'a L,
	det: Details,
	scale: f32,
}
impl<'a, L: Layable> RootContext<'a, L> {
	pub fn new(layable: &'a L, det: Details, scale: f32) -> Self {
		RootContext {
			layable,
			det,
			scale,
		}
	}

	pub fn render(&self, d: &mut RaylibDrawHandle) {
		self.layable.render(d, self.det, self.scale);
	}
	pub fn handle_input(&self, rl: &mut RaylibHandle) -> impl Iterator<Item = Event> {
		let mouse_back = if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
			let (ptr_x, ptr_y) = (rl.get_mouse_x(), rl.get_mouse_y());

			if ptr_x as f32 > self.det.x as f32 && ptr_y as f32 > self.det.y as f32 {
				self.layable.pass_event(
					Event::MouseEvent { x: ptr_x, y: ptr_y },
					self.det,
					self.scale,
				)
			} else {
				None
			}
		} else {
			None
		};

		mouse_back.into_iter()
	}
}
