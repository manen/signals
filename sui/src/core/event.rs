#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Event {
	MouseEvent(MouseEvent),
	KeyboardEvent(crate::form::UniqueId, KeyboardEvent),
}
impl Event {
	pub fn ret<T: 'static>(ret: T) -> ReturnEvent {
		ReturnEvent::new(ret)
	}
}
#[derive(Copy, Clone, Debug, PartialEq)]
/// mouseevent can figure out which component to go to from the coords and the det passed to `pass_event`
pub enum MouseEvent {
	// these all use window coords
	MouseClick { x: i32, y: i32 },
	MouseHeld { x: i32, y: i32 },
	MouseRelease { x: i32, y: i32 },

	Scroll { x: i32, y: i32, amount: f32 },
}
impl MouseEvent {
	pub fn at(&self) -> (i32, i32) {
		match self {
			&Self::MouseClick { x, y } => (x, y),
			&Self::MouseHeld { x, y } => (x, y),
			&Self::MouseRelease { x, y } => (x, y),
			&Self::Scroll { x, y, amount: _ } => (x, y),
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeyboardEvent {
	CharPressed(char),
	// this is enough for now
}

// -

pub type DialogCommand = crate::dialog::Command;
pub type FocusCommand = crate::form::FocusCommand;
pub type TypeCommand = crate::form::typable::TypeEvent;

/// FeaturedReturn is automatically implemented for any type that can be formed from any
/// builtin sui returnevent type
pub trait FeaturedReturn:
	From<DialogCommand> + From<FocusCommand> + From<TypeCommand> + 'static
{
	fn cast_event(event: ReturnEvent) -> ReturnEvent {
		if event.can_take::<DialogCommand>() {
			if let Some(dialog) = event.take::<DialogCommand>() {
				return Event::ret(Self::from(dialog));
			} else {
				unreachable!()
			}
		} else if event.can_take::<FocusCommand>() {
			if let Some(form) = event.take::<FocusCommand>() {
				return Event::ret(Self::from(form));
			} else {
				unreachable!()
			}
		} else if event.can_take::<TypeCommand>() {
			if let Some(typ) = event.take::<TypeCommand>() {
				return Event::ret(Self::from(typ));
			} else {
				unreachable!()
			}
		} else {
			return event;
		}
	}
}
impl<T> FeaturedReturn for T where
	T: From<DialogCommand> + From<FocusCommand> + From<TypeCommand> + 'static
{
}

use std::any::Any;

#[derive(Debug)]
pub struct ReturnEvent {
	boxed: Box<dyn Any>,
}
impl ReturnEvent {
	pub fn new<T: 'static>(event: T) -> Self {
		Self {
			boxed: Box::new(event),
		}
	}

	pub fn can_take<T: 'static>(&self) -> bool {
		self.boxed.downcast_ref::<T>().is_some()
	}
	pub fn take<T: 'static>(self) -> Option<T> {
		self.boxed.downcast::<T>().ok().map(|a| *a)
	}
}
