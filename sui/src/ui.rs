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
	comp::Div::new(false, components.into()).into_comp()
}
pub fn page_h<'a>(components: impl Into<Vec<Comp<'a>>>) -> Comp<'a> {
	comp::Div::new(true, components.into()).into_comp()
}
pub fn text<'a, T: Into<Cow<'a, str>>>(text: T, size: i32) -> Comp<'a> {
	comp::Text::new(text, size).into_comp()
}

macro_rules! handle_input_impl {
	($self:expr,$rl:expr) => {{
		let (ptr_x, ptr_y) = ($rl.get_mouse_x(), $rl.get_mouse_y());

		let mouse_left_pressed = if $rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
			if ptr_x as f32 > $self.det.x as f32 && ptr_y as f32 > $self.det.y as f32 {
				$self.layable.pass_event(
					Event::MouseClick { x: ptr_x, y: ptr_y },
					$self.det,
					$self.scale,
				)
			} else {
				None
			}
		} else {
			None
		};
		let mouse_left_down = if $rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
			if ptr_x as f32 > $self.det.x as f32 && ptr_y as f32 > $self.det.y as f32 {
				$self.layable.pass_event(
					Event::MouseHeld { x: ptr_x, y: ptr_y },
					$self.det,
					$self.scale,
				)
			} else {
				None
			}
		} else {
			None
		};
		let mouse_left_released = if $rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
			if ptr_x as f32 > $self.det.x as f32 && ptr_y as f32 > $self.det.y as f32 {
				$self.layable.pass_event(
					Event::MouseRelease { x: ptr_x, y: ptr_y },
					$self.det,
					$self.scale,
				)
			} else {
				None
			}
		} else {
			None
		}
		.into_iter();

		let mouse_left_pressed = mouse_left_pressed.into_iter();
		let mouse_left_down = mouse_left_down.into_iter();
		let mouse_left_released = mouse_left_released.into_iter();

		mouse_left_pressed
			.chain(mouse_left_down)
			.chain(mouse_left_released)
	}};
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
		handle_input_impl!(self, rl)
	}
	/// duplcate of [Self::handle_input] with a different raylib handle
	pub fn handle_input_d(&self, d: &mut RaylibDrawHandle) -> impl Iterator<Item = Event> {
		handle_input_impl!(self, d)
	}
}
