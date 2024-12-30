use std::borrow::Cow;

use raylib::ffi::MouseButton;

use crate::{
	comp::{self, Comp, Compatible},
	core::Event,
	Layable,
};

pub fn page<'a>(components: impl Into<Vec<Comp<'a>>>) -> Comp<'a> {
	comp::Page::new(components, false).into_comp()
}
pub fn page_h<'a>(components: impl Into<Vec<Comp<'a>>>) -> Comp<'a> {
	comp::Page::new(components, true).into_comp()
}
pub fn text<'a, T: Into<Cow<'a, str>>>(text: T, size: i32) -> Comp<'a> {
	comp::Text::new(text, size).into_comp()
}

// only temporary api on paper
pub fn handle_input<'a, C: Layable>(
	comp: &'a C,
	rl: &mut raylib::RaylibHandle,
	base_x: i32,
	base_y: i32,
	scale: f32,
) -> Option<crate::core::Event> {
	if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
		let (ptr_x, ptr_y) = (rl.get_mouse_x(), rl.get_mouse_y());

		if ptr_x as f32 > base_x as f32 && ptr_y as f32 > base_y as f32 {
			comp.pass_event(Event::MouseEvent {
				x: ((ptr_x - base_x) as f32 / scale) as i32,
				y: ((ptr_y - base_y) as f32 / scale) as i32,
			})
		} else {
			None
		}
	} else {
		None
	}
}
pub fn render_root<'a, C: Layable>(
	comp: &'a C,
	d: &mut raylib::drawing::RaylibDrawHandle,
	x: i32,
	y: i32,
	scale: f32,
) {
	comp.render(
		d,
		crate::Details {
			x,
			y,
			aw: -1,
			ah: -1,
		},
		scale,
	);
}
