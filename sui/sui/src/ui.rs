use std::{borrow::Cow, ops::DerefMut};

use raylib::{
	ffi::{KeyboardKey, MouseButton},
	prelude::RaylibDrawHandle,
	RaylibHandle,
};

use crate::{
	comp::{
		self,
		scrollable::{ScrollableMode, ScrollableState},
		Comp, Compatible,
	},
	core::{Event, FeaturedReturn, ReturnEvent, Store},
	form::FocusHandler,
	Details, Layable,
};

/// this will turn any layable into a Comp \
///! do not use if Comp has an enum variant for the layable you're using,
///  as it will create a DynamicLayable for nothing (causing performance & memory overhead)
pub fn custom<'a, L: Layable + std::fmt::Debug + Clone + 'a>(layable: L) -> Comp<'a> {
	crate::DynamicLayable::new(layable).into_comp()
}

pub fn div<D: comp::div::DivComponents>(components: D) -> comp::Div<D> {
	comp::Div::new(false, false, components)
}
pub fn div_h<D: comp::div::DivComponents>(components: D) -> comp::Div<D> {
	comp::Div::new(true, false, components)
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

	pub fn render(&self, d: &mut crate::Handle) {
		self.layable.render(d, self.det, self.scale);
	}
	pub fn handle_input<'b, E: FeaturedReturn, H: DerefMut<Target = RaylibHandle>>(
		&'b self,
		rl: &mut H,
		focus: &FocusHandler,
	) -> impl Iterator<Item = Result<E, ReturnEvent>> + 'b {
		use crate::core::KeyboardEvent;
		use crate::core::MouseEvent;

		let (ptr_x, ptr_y) = (rl.get_mouse_x(), rl.get_mouse_y());

		let mouse_events = if ptr_x as f32 > self.det.x as f32 && ptr_y as f32 > self.det.y as f32 {
			let mouse_left_pressed = if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
				Some(Event::MouseEvent(MouseEvent::MouseClick {
					x: ptr_x,
					y: ptr_y,
				}))
			} else {
				None
			};
			let mouse_left_down = if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
				Some(Event::MouseEvent(MouseEvent::MouseHeld {
					x: ptr_x,
					y: ptr_y,
				}))
			} else {
				None
			};
			let mouse_left_released = if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT)
			{
				Some(Event::MouseEvent(MouseEvent::MouseRelease {
					x: ptr_x,
					y: ptr_y,
				}))
			} else {
				None
			};

			let mouse_wheel_move = rl.get_mouse_wheel_move();
			let mouse_wheel = if mouse_wheel_move != 0.0 {
				Some(Event::MouseEvent(MouseEvent::Scroll {
					x: ptr_x,
					y: ptr_y,
					amount: mouse_wheel_move,
				}))
			} else {
				None
			};

			mouse_left_pressed
				.into_iter()
				.chain(mouse_left_down)
				.chain(mouse_left_released)
				.chain(mouse_wheel)
		} else {
			None.into_iter().chain(None).chain(None).chain(None)
		};

		let keyboard_events = {
			let key = rl.get_char_pressed();
			let key = match key {
				Some(a) => Some(a),
				None => {
					if rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE) {
						Some(crate::form::typable::BACKSPACE)
					} else if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
						Some('\n')
					} else {
						None
					}
				}
			};
			key.map(|key| {
				Some(Event::KeyboardEvent(
					focus.get(),
					KeyboardEvent::CharPressed(key),
				))
			})
			.flatten()
		};

		mouse_events
			.chain(keyboard_events)
			.map(|event| {
				self.layable
					.pass_event(event, self.det, self.scale)
					.map(|event| {
						let cast = E::cast_event(event);
						if cast.can_take::<E>() {
							Ok(cast.take().expect("can_take returned true, taking failed"))
						} else {
							Err(cast)
						}
					})
			})
			.filter_map(|a| a)
	}
}

/// LayableExt provides associated functions for most comp::*::new calls
pub trait LayableExt: Layable + Sized {
	/// see [comp::Centered]
	fn centered(self) -> comp::Centered<Self> {
		comp::Centered::new(self)
	}

	/// see [comp::AtEnd]
	fn to_right(self) -> comp::AtEnd<Self> {
		comp::AtEnd::to_right(self)
	}
	/// see [comp::AtEnd]
	fn to_bottom(self) -> comp::AtEnd<Self> {
		comp::AtEnd::to_bottom(self)
	}
	/// see [comp::AtEnd]
	fn to_bottom_right(self) -> comp::AtEnd<Self> {
		comp::AtEnd::to_bottom_right(self)
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

	/// see [comp::Scale]
	fn scale(self, scale: f32) -> comp::Scale<Self> {
		comp::Scale::new(self, scale)
	}

	/// see [comp::Margin]
	fn margin(self, margin: i32) -> comp::Margin<Self> {
		comp::Margin::all(margin, self)
	}
	/// see [comp::Margin]
	fn margin_v(self, margin: i32) -> comp::Margin<Self> {
		comp::Margin::vertical(margin, self)
	}
	/// see [comp::Margin]
	fn margin_h(self, margin: i32) -> comp::Margin<Self> {
		comp::Margin::horizontal(margin, self)
	}

	/// see [comp::Scrollable]
	fn scrollable_vert(self, state: Store<ScrollableState>) -> comp::Scrollable<comp::Crop<Self>> {
		comp::Scrollable::new(state, ScrollableMode::Vertical, self)
	}
	/// see [comp::Scrollable]
	fn scrollable_horiz(self, state: Store<ScrollableState>) -> comp::Scrollable<comp::Crop<Self>> {
		comp::Scrollable::new(state, ScrollableMode::Horizontal, self)
	}
	/// see [comp::Scrollable]
	fn scrollable(self, state: Store<ScrollableState>) -> comp::Scrollable<comp::Crop<Self>> {
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

	/// the context from which root components should be interacted with \
	/// see [RootContext]
	fn root_context(&self, det: Details, scale: f32) -> RootContext<Self> {
		RootContext::new(self, det, scale)
	}
}
impl<L: Layable> LayableExt for L {}
