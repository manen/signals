use raylib::{
	prelude::{RaylibDraw, RaylibDrawHandle},
	RaylibHandle,
};

use crate::{consts, gfx::ui};

pub struct SelectBar<'a, T: Clone + PartialEq> {
	list: &'a [(&'a str, T)],
}
impl<'a, T: Clone + PartialEq> SelectBar<'a, T> {
	pub fn new(list: &'a [(&'a str, T)]) -> Self {
		Self { list }
	}

	/// returns whether the select bar was used in this tick
	pub fn tick(&self, rl: &mut RaylibHandle, det: ui::Details, select: &mut T) -> bool {
		let mouse = (rl.get_mouse_x(), rl.get_mouse_y());
		let mouse = (mouse.0 - det.x, mouse.1 - det.y);
		for (i, edet) in det.split_v(self.list.len() as i32).enumerate() {
			if mouse.0 >= edet.x && mouse.0 <= edet.x + edet.aw // x
			&& mouse.1 >= edet.y && mouse.1 <= edet.y + edet.ah
			// y
			{
				if rl.is_mouse_button_pressed(consts::TOOL_USE) {
					*select = self.list[i].1.clone();
					return true;
				}
				if rl.is_mouse_button_down(consts::TOOL_USE) {
					return true;
				}
			}
		}
		false
	}
	pub fn render(&self, d: &mut RaylibDrawHandle, det: ui::Details, selected: Option<&T>) {
		for (edet, (name, opt)) in det.split_v(self.list.len() as i32).zip(self.list) {
			let is_selected = selected.map(|x| x == opt).unwrap_or(false);
			d.draw_text(
				name,
				edet.x,
				edet.y,
				16,
				if is_selected {
					consts::SELECT_BAR_SELECTED
				} else {
					consts::SELECT_BAR_UNSELECTED
				},
			);
		}
	}
}
