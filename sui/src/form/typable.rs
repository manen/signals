use crate::{
	core::{Event, KeyboardEvent, Store},
	Layable, Text,
};

use super::UniqueId;

pub const BACKSPACE: char = '\x08';

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeEvent {
	Handled,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypableData {
	pub uid: UniqueId,
	pub text: String,
}
impl TypableData {
	pub fn new() -> Self {
		Self {
			uid: UniqueId::new(),
			text: String::new(),
		}
	}
}

#[derive(Clone, Debug)]
/// this component is not a fully featured textbox. \
/// it just renders a flashing pointer and the text currently being written.
///
/// you can take out the text written by reading the store passed to [Typable::new]
pub struct Typable {
	store: Store<TypableData>,
	text_size: i32,
}
impl Typable {
	pub fn default(text_size: i32) -> Self {
		Self::new(Store::new(TypableData::new()), text_size)
	}
	pub fn new(store: Store<TypableData>, text_size: i32) -> Self {
		Self { store, text_size }
	}

	pub fn with_text<T>(&self, f: impl FnOnce(Text) -> T) -> T {
		self.store
			.with_borrow(|data| f(Text::new(&data.text, self.text_size)))
	}
}
impl Layable for Typable {
	fn size(&self) -> (i32, i32) {
		self.with_text(|a| a.size())
	}
	fn render(&self, d: &mut raylib::prelude::RaylibDrawHandle, det: crate::Details, scale: f32) {
		self.with_text(|a| a.render(d, det, scale))
	}
	fn pass_event(
		&self,
		event: crate::core::Event,
		_det: crate::Details,
		_scale: f32,
	) -> Option<crate::core::ReturnEvent> {
		let self_uiq = self.store.with_borrow(|a| a.uid);
		match event {
			Event::KeyboardEvent(this_uiq, KeyboardEvent::CharPressed(key))
				if this_uiq == self_uiq =>
			{
				self.store.with_mut_borrow(|data| match key {
					BACKSPACE => {
						data.text.pop();
					}
					_ => data.text.push(key),
				});
				Some(Event::ret(TypeEvent::Handled))
			}
			_ => None,
		}
	}
}
