use raylib::{color::Color, prelude::RaylibDraw};

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
	pub fn with_default(default: String) -> Self {
		Self {
			uid: UniqueId::new(),
			text: default,
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
	fn render(&self, d: &mut crate::Handle, det: crate::Details, scale: f32) {
		self.with_text(|a| a.render(d, det, scale));

		if d.focus() == self.store.with_borrow(|data| data.uid) {
			let should_draw_blinker = d.get_time() * 2.0;
			let should_draw_blinker = should_draw_blinker - should_draw_blinker.floor();

			if should_draw_blinker < 0.5 {
				d.draw_rectangle(
					det.x,
					det.y,
					(3.0 * scale) as i32,
					(self.text_size as f32 * scale) as i32,
					Color::WHITE,
				);
			}
		}
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
