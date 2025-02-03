use std::ops::{Deref, DerefMut};

use raylib::prelude::RaylibDrawHandle;

use crate::form::{FocusHandler, UniqueId};

pub struct Handle<'a> {
	d: RaylibDrawHandle<'a>,
	focus: UniqueId,
}
impl<'a> Handle<'a> {
	pub fn new(d: RaylibDrawHandle<'a>, fh: &FocusHandler) -> Self {
		Self {
			d,
			focus: fh.with_borrow(|a| *a),
		}
	}
	pub fn new_unfocused(d: RaylibDrawHandle<'a>) -> Self {
		Self {
			d,
			focus: UniqueId::null(),
		}
	}

	pub fn focus(&self) -> UniqueId {
		self.focus
	}
	pub fn take(self) -> RaylibDrawHandle<'a> {
		self.d
	}
}
impl<'a> Deref for Handle<'a> {
	type Target = RaylibDrawHandle<'a>;

	fn deref(&self) -> &Self::Target {
		&self.d
	}
}
impl<'a> DerefMut for Handle<'a> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.d
	}
}
