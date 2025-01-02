mod dyn_layable;
pub use dyn_layable::DynamicLayable;
use raylib::prelude::RaylibDrawHandle;
use std::fmt::Debug;

pub trait Layable {
	fn size(&self) -> (i32, i32);
	fn render(&self, d: &mut RaylibDrawHandle, det: Details, scale: f32);

	/// this function is called by the parent of this component \
	/// return events to be bubbled back \
	fn pass_event(&self, event: Event, det: Details, scale: f32) -> Option<Event>;
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Event {
	/// uses window coords
	MouseEvent { x: i32, y: i32 },

	/// use these to bubble
	Named {
		/// id is meant to be a general identifier of what this event's about
		id: &'static str,
		/// n could be anything you want, probably most useful as an array index
		n: i32,
	},
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Details {
	pub x: i32,
	pub y: i32,
	/// available width
	pub aw: i32,
	/// available height
	pub ah: i32,
}
impl Details {
	pub fn new(x: i32, y: i32, aw: i32, ah: i32) -> Self {
		Self { x, y, aw, ah }
	}
	pub fn window(w: i32, h: i32) -> Self {
		Self::new(0, 0, w, h)
	}

	pub fn from_top(&self, h: i32) -> Self {
		Self {
			x: self.x,
			y: self.y,
			aw: self.aw,
			ah: h,
		}
	}
	pub fn from_bottom(&self, h: i32) -> Self {
		Self {
			x: self.x,
			y: self.y + self.ah - h,
			aw: self.aw,
			ah: h,
		}
	}
	pub fn from_left(&self, w: i32) -> Self {
		Self {
			x: self.x,
			y: self.y,
			aw: w,
			ah: self.ah,
		}
	}
	pub fn from_right(&self, w: i32) -> Self {
		Self {
			x: self.x + self.aw - w,
			y: self.y,
			aw: w,
			ah: self.ah,
		}
	}

	pub fn split_v(&self, pieces: i32) -> impl Iterator<Item = Self> {
		let one_w = self.aw / pieces;
		let y = self.y;
		let ah = self.ah;

		(0..pieces).map(move |i| one_w * i).map(move |x| Self {
			x,
			y,
			aw: one_w,
			ah,
		})
	}
	pub fn split_h(&self, pieces: i32) -> impl Iterator<Item = Self> {
		let one_h = self.ah / pieces;
		let x = self.x;
		let aw = self.aw;

		(0..pieces).map(move |i| one_h * i).map(move |y| Self {
			x,
			y,
			aw,
			ah: one_h,
		})
	}

	pub fn mul_size(self, scale: f32) -> Self {
		Self {
			aw: (self.aw as f32 * scale) as _,
			ah: (self.ah as f32 * scale) as _,
			..self
		}
	}
	pub fn is_inside(&self, x: i32, y: i32) -> bool {
		x >= self.x && x <= self.x + self.aw // x
			&& y >= self.y && y <= self.y + self.ah
	}
}
