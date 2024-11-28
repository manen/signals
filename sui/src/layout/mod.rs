use crate::Details;
use raylib::prelude::RaylibDrawHandle;

pub mod page;
pub use page::Page;

pub trait Layable {
	fn size(&self) -> (i32, i32);
	fn render(&self, d: &mut RaylibDrawHandle, det: Details, scale: f32);

	/// this function is called by the parent of this component \
	/// return events to be bubbled back
	fn pass_event(&self, event: Event) -> Option<Event>;
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Event {
	MouseEvent {
		x: i32,
		y: i32,
	},

	/// use these to bubble
	Named {
		/// id is meant to be a general identifier of what this event's about
		id: &'static str,
		/// n could be anything you want, probably most useful as an array index
		n: i32,
	},
}
