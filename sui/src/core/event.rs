#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Event {
	// these all use window coords
	MouseClick { x: i32, y: i32 },
	MouseHeld { x: i32, y: i32 },
	MouseRelease { x: i32, y: i32 },
}
impl Event {
	pub fn ret<T: 'static>(ret: T) -> ReturnEvent {
		ReturnEvent::new(ret)
	}
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
