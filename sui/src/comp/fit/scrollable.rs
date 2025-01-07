use raylib::prelude::RaylibDraw;

use crate::{core::Store, Layable};

pub const SCROLLBAR_WIDTH: f32 = 10.0; // it's getting multiplied by scale anyway so we just savin a step
const SCROLLBAR_BG_COLOR: raylib::color::Color = crate::comp::select_bar::color(33, 35, 38, 255);
const SCROLLBAR_HANDLE_COLOR: raylib::color::Color =
	crate::comp::select_bar::color(106, 113, 122, 255);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScrollableMode {
	Neither,
	Vertical,
	Horizontal,
	Both,
}
impl ScrollableMode {
	fn multipliers(&self) -> (i32, i32) {
		match self {
			ScrollableMode::Neither => (0, 0),
			ScrollableMode::Vertical => (0, 1),
			ScrollableMode::Horizontal => (1, 0),
			ScrollableMode::Both => (1, 1),
		}
	}
	fn multipliers_f32(&self) -> (f32, f32) {
		let (x, y) = self.multipliers();
		(x as f32, y as f32)
	}
	fn bools(&self) -> (bool, bool) {
		match self {
			ScrollableMode::Neither => (false, false),
			ScrollableMode::Vertical => (false, true),
			ScrollableMode::Horizontal => (true, false),
			ScrollableMode::Both => (true, true),
		}
	}
}

/// ScrollableData stores the data needed for a scrollable to actually scroll
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ScrollableState {
	// unscaled pixels scrolled in each direction, from 0 to big_width/big_height-small_width/small_height
	pub scroll_x: i32,
	pub scroll_y: i32,
}

#[derive(Clone, Debug)]
pub struct Scrollable<L: Layable> {
	state: Store<ScrollableState>,
	mode: ScrollableMode,
	layable: L,
}
impl<L: Layable> Scrollable<L> {
	pub fn new(state: Store<ScrollableState>, mode: ScrollableMode, layable: L) -> Self {
		Self {
			state,
			mode,
			layable,
		}
	}

	fn view(&self, scale: f32) -> View<&L> {
		let (scroll_x, scroll_y) = self.state.with_borrow(|a| (a.scroll_x, a.scroll_y));

		View::new(
			&self.layable,
			(scroll_x as f32 * scale) as i32,
			(scroll_y as f32 * scale) as i32,
		)
	}
	/// the det view is rendered with
	fn l_det(&self, det: crate::Details, scale: f32) -> crate::Details {
		let (x_mul, y_mul) = self.mode.multipliers_f32();

		crate::Details {
			x: det.x,
			y: det.y,
			aw: det.aw - (x_mul * SCROLLBAR_WIDTH * scale) as i32,
			ah: det.ah - (y_mul * SCROLLBAR_WIDTH * scale) as i32,
		}
	}
}
impl<L: Layable> Layable for Scrollable<L> {
	/// returns self.layable.size(), because this scrollable is likely in a FixedSize, so it doesn't really matter
	fn size(&self) -> (i32, i32) {
		self.layable.size()
	}
	fn render(&self, d: &mut raylib::prelude::RaylibDrawHandle, det: crate::Details, scale: f32) {
		let (l_w, l_h) = self.layable.size();

		let view = self.view(scale);
		let view_det = self.l_det(det, scale);
		view.render(d, view_det, scale);

		let (scrollbar_at_side, scrollbar_at_bottom) = self.mode.bools();

		if scrollbar_at_side {
			let (scrollbar_base_x, scrollbar_base_y) = (view_det.x + view_det.aw, view_det.y);
			let (scrollbar_w, scrollbar_h) = ((SCROLLBAR_WIDTH * scale) as i32, view_det.ah);
			d.draw_rectangle(
				scrollbar_base_x,
				scrollbar_base_y,
				scrollbar_w,
				scrollbar_h,
				SCROLLBAR_BG_COLOR,
			);

			let (_, scroll_y) = self.state.with_borrow(|a| (a.scroll_x, a.scroll_y));

			let (scrollbar_handle_base_x, scrollbar_handle_base_y) = (
				scrollbar_base_x,
				scrollbar_base_y + (scroll_y as f32 / l_h as f32 * det.ah as f32) as i32,
			);
			let (scrollbar_handle_w, scrollbar_handle_h) = (
				(SCROLLBAR_WIDTH * scale) as i32,
				(l_h as f32 / det.ah as f32 * scale) as i32,
			);
			d.draw_rectangle(
				scrollbar_handle_base_x,
				scrollbar_handle_base_y,
				scrollbar_handle_w,
				scrollbar_handle_h,
				SCROLLBAR_HANDLE_COLOR,
			);
		}
		if scrollbar_at_bottom {
			let (scrollbar_base_x, scrollbar_base_y) = (view_det.x, view_det.y + view_det.ah);
			let (scrollbar_w, scrollbar_h) = (view_det.aw, (SCROLLBAR_WIDTH * scale) as i32);
			d.draw_rectangle(
				scrollbar_base_x,
				scrollbar_base_y,
				scrollbar_w,
				scrollbar_h,
				SCROLLBAR_BG_COLOR,
			);

			let (scroll_x, _) = self.state.with_borrow(|a| (a.scroll_x, a.scroll_y));

			let (scrollbar_handle_base_x, scrollbar_handle_base_y) = (
				scrollbar_base_x + (scroll_x as f32 / l_w as f32 * det.aw as f32) as i32,
				scrollbar_base_y,
			);
			let (scrollbar_handle_w, scrollbar_handle_h) = (
				// ((l_w as f32 - det.aw as f32) / l_w as f32 * det.aw as f32 * scale) as i32,
				(SCROLLBAR_WIDTH * scale * 4.0) as i32,
				(SCROLLBAR_WIDTH * scale) as i32,
			);
			d.draw_rectangle(
				scrollbar_handle_base_x,
				scrollbar_handle_base_y,
				scrollbar_handle_w,
				scrollbar_handle_h,
				SCROLLBAR_HANDLE_COLOR,
			);
		}
	}
	fn pass_event(
		&self,
		event: crate::core::Event,
		det: crate::Details,
		scale: f32,
	) -> Option<crate::core::Event> {
		let view = self.view(scale);
		view.pass_event(event, self.l_det(det, scale), scale)
	}
}

#[derive(Clone, Debug)]
/// Renders `self.layable`, with an offset that it will appear as though `self.layable` is rendering from `(self.base_x, self.base_y)`
///
/// does not currently crop the content. this whole struct is basically just to make making scrollables crop their content easier
pub struct View<L: Layable> {
	layable: L,
	base_x: i32,
	base_y: i32,
}
impl<L: Layable> View<L> {
	pub fn new(layable: L, x: i32, y: i32) -> Self {
		Self {
			layable,
			base_x: x,
			base_y: y,
		}
	}
	pub fn take(self) -> L {
		self.layable
	}

	pub fn l_det(&self, det: crate::Details, _scale: f32) -> crate::Details {
		crate::Details {
			x: det.x - self.base_x,
			y: det.y - self.base_y,
			aw: det.aw + self.base_x,
			ah: det.ah + self.base_y,
		}
	}
}
impl<L: Layable> Layable for View<L> {
	fn size(&self) -> (i32, i32) {
		self.layable.size()
	}
	fn render(&self, d: &mut raylib::prelude::RaylibDrawHandle, det: crate::Details, scale: f32) {
		self.layable.render(d, self.l_det(det, scale), scale);
	}
	fn pass_event(
		&self,
		event: crate::core::Event,
		det: crate::Details,
		scale: f32,
	) -> Option<crate::core::Event> {
		self.layable
			.pass_event(event, self.l_det(det, scale), scale)
	}
}
