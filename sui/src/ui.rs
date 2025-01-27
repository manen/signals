use std::borrow::Cow;

use raylib::{ffi::MouseButton, prelude::RaylibDrawHandle, RaylibHandle};

use crate::{
	comp::{
		self,
		scrollable::{ScrollableMode, ScrollableState},
		Comp, Compatible,
	},
	core::{Event, ReturnEvent, Store},
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
	pub fn handle_input(&self, rl: &mut RaylibHandle) -> impl Iterator<Item = ReturnEvent> {
		handle_input_impl!(self, rl)
	}
	/// duplcate of [Self::handle_input] with a different raylib handle
	pub fn handle_input_d(&self, d: &mut RaylibDrawHandle) -> impl Iterator<Item = ReturnEvent> {
		handle_input_impl!(self, d)
	}
}

/// LayableExt provides associated functions for most comp::*::new calls
pub trait LayableExt: Layable + Sized {
	/// see [comp::Centered]
	fn centered(self) -> comp::Centered<Self> {
		comp::Centered::new(self)
	}
	/// see [comp::Crop]
	fn crop(self) -> comp::Crop<Self> {
		comp::Crop::new(self)
	}

	/// see [comp::FixedSize]
	fn fix_w(self, width: i32) -> comp::FixedSize<Self> {
		comp::FixedSize::fix_w(width, self)
	}
	/// see [comp::FixedSize]
	fn fix_h(self, height: i32) -> comp::FixedSize<Self> {
		comp::FixedSize::fix_h(height, self)
	}
	/// see [comp::FixedSize]
	fn fix_wh(self, width: i32, height: i32) -> comp::FixedSize<Self> {
		comp::FixedSize::fix_size((width, height), self)
	}
	/// see [comp::FixedSize]
	fn fix_wh_square(self, both: i32) -> comp::FixedSize<Self> {
		comp::FixedSize::fix_both(both, self)
	}

	/// see [comp::ScaleToFit]
	fn scale_h_to_fix(self, fix_width: i32) -> comp::ScaleToFit<Self> {
		comp::ScaleToFit::fix_w(fix_width, self)
	}
	/// see [comp::ScaleToFit]
	fn scale_w_to_fix(self, fix_height: i32) -> comp::ScaleToFit<Self> {
		comp::ScaleToFit::fix_h(fix_height, self)
	}

	/// see [comp::Margin]
	fn margin(self, margin: i32) -> comp::Margin<Self> {
		comp::Margin::all(margin, self)
	}

	/// see [comp::Scrollable]
	fn scrollable_vert(self, state: Store<ScrollableState>) -> comp::Crop<comp::Scrollable<Self>> {
		comp::Scrollable::new(state, ScrollableMode::Vertical, self)
	}
	/// see [comp::Scrollable]
	fn scrollable_horiz(self, state: Store<ScrollableState>) -> comp::Crop<comp::Scrollable<Self>> {
		comp::Scrollable::new(state, ScrollableMode::Horizontal, self)
	}
	/// see [comp::Scrollable]
	fn scrollable(self, state: Store<ScrollableState>) -> comp::Crop<comp::Scrollable<Self>> {
		comp::Scrollable::new(state, ScrollableMode::Both, self)
	}

	/// see [comp::Clickable]
	fn clickable<T: Clone + 'static, F: Fn((i32, i32)) -> T>(
		self,
		gen_ret: F,
	) -> comp::Clickable<Self, F, T> {
		comp::Clickable::new(gen_ret, self)
	}
	/// see [comp::Clickable]
	fn clickable_fallback<T: Clone + 'static, F: Fn((i32, i32)) -> T>(
		self,
		gen_ret: F,
	) -> comp::Clickable<Self, F, T> {
		comp::Clickable::new_fallback(gen_ret, self)
	}

	/// see [comp::Debug]
	fn debug(self) -> comp::Debug<Self> {
		comp::Debug::new(self)
	}

	/// see [comp::Overlay]
	fn overlay<L1: Layable>(self, foreground: L1) -> comp::Overlay<L1, Self> {
		comp::Overlay::new(self, foreground)
	}
	/// see [comp::Overlay]
	fn with_background<L1: Layable>(self, background: L1) -> comp::Overlay<Self, L1> {
		comp::Overlay::new(background, self)
	}
}
impl<L: Layable> LayableExt for L {}
