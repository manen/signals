use std::{borrow::Cow, ops::DerefMut};

use raylib::{color::Color, math::Ray, prelude::RaylibDraw, RaylibHandle, RaylibThread};
use sui::{
	core::{ReturnEvent, Store},
	form::UniqueId,
	Layable, LayableExt, RootContext,
};

#[derive(Debug)]
pub enum Event {
	Focus(sui::core::FocusCommand),
	Dialog(sui::core::DialogCommand),
	Type(sui::core::TypeCommand),
	Other(ReturnEvent),
}
macro_rules! from {
	($var:ident, $ty:ty) => {
		impl From<$ty> for Event {
			fn from(value: $ty) -> Self {
				Event::$var(value)
			}
		}
	};
}
from!(Focus, sui::core::FocusCommand);
from!(Dialog, sui::core::DialogCommand);
from!(Type, sui::core::TypeCommand);

#[derive(Debug)]
pub struct Context<L: Layable> {
	pub l: L,
	pub rl: RaylibHandle,
	pub thread: RaylibThread,
}
impl<L: Layable> Context<L> {
	pub fn new(layable: L, rl: RaylibHandle, thread: RaylibThread) -> Self {
		Self {
			l: layable,
			rl,
			thread,
		}
	}

	/// starts rendering, contains the main loop \
	/// to use your own main loop, call [Self::tick] in a loop
	pub fn start(&mut self) {
		let mut focus = Store::new(UniqueId::null());

		while !self.rl.window_should_close() {
			self.tick(&mut focus);
		}
	}
	pub fn tick(&mut self, focus: &mut Store<UniqueId>) {
		let screen = sui::Details::rl_window(&self.rl);
		let ctx = self.l.root_context(screen, 1.0);

		let mut r = &mut self.rl;
		let r = &mut r;

		for event in ctx.handle_input(r, focus) {
			match event {
				Ok(Event::Focus(cmd)) => cmd.apply(focus),
				_ => eprintln!("dropped event {event:?}"),
			}
		}

		let mut d = self.rl.begin_drawing(&self.thread);
		d.clear_background(Color::BLACK);

		let mut d = sui::Handle::new(d, focus);

		ctx.render(&mut d);
	}
}
