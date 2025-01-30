#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Event {
	MouseEvent(MouseEvent),
	KeyboardEvent(crate::form::UniqueId, KeyboardEvent),
}
impl Event {
	pub fn ret<T: 'static>(ret: T) -> ReturnEvent {
		ReturnEvent::new(ret)
	}
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// mouseevent can figure out which component to go to from the coords and the det passed to `pass_event`
pub enum MouseEvent {
	// these all use window coords
	MouseClick { x: i32, y: i32 },
	MouseHeld { x: i32, y: i32 },
	MouseRelease { x: i32, y: i32 },
}
impl MouseEvent {
	pub fn at(&self) -> (i32, i32) {
		match self {
			&Self::MouseClick { x, y } => (x, y),
			&Self::MouseHeld { x, y } => (x, y),
			&Self::MouseRelease { x, y } => (x, y),
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeyboardEvent {
	CharPressed(char),
	// this is enough for now
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

	pub fn take<T: 'static>(self) -> Option<T> {
		self.boxed.downcast::<T>().ok().map(|a| *a)
	}
}
