use raylib::{
	ffi::MouseButton,
	prelude::{RaylibDraw, RaylibDrawHandle},
	RaylibHandle,
};

use crate::color;
use raylib::color::Color;
pub const SELECT_BAR_SELECTED: Color = color(240, 240, 240, 255);
pub const SELECT_BAR_UNSELECTED: Color = color(160, 160, 160, 255);

use crate::Details;

pub struct SelectBar<'a, T: Clone + PartialEq> {
	list: &'a [(&'a str, T)],
}
impl<'a, T: Clone + PartialEq> SelectBar<'a, T> {
	pub fn new(list: &'a [(&'a str, T)]) -> Self {
		Self { list }
	}

	/// returns whether the select bar was used in this tick
	pub fn tick(&self, rl: &mut RaylibHandle, det: Details, select: &mut T) -> bool {
		let mouse = (rl.get_mouse_x(), rl.get_mouse_y());

		for (i, edet) in det.split_v(self.list.len() as i32).enumerate() {
			if edet.is_inside(mouse.0, mouse.1) {
				if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
					*select = self.list[i].1.clone();
					return true;
				}
				if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
					return true;
				}
			}
		}
		false
	}
	pub fn render(&self, d: &mut crate::Handle, det: Details, selected: Option<&T>) {
		for (edet, (name, opt)) in det.split_v(self.list.len() as i32).zip(self.list) {
			let is_selected = selected.map(|x| x == opt).unwrap_or(false);
			d.draw_text(
				name,
				edet.x,
				edet.y,
				16,
				if is_selected {
					SELECT_BAR_SELECTED
				} else {
					SELECT_BAR_UNSELECTED
				},
			);
		}
	}
}
