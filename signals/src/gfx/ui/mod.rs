mod select_bar;
pub use select_bar::SelectBar;

#[derive(Copy, Clone, Debug)]
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
	pub fn screen(w: i32, h: i32) -> Self {
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
}