use crate::{
	comp::{self, Comp, Compatible},
	layout, Layable,
};

pub fn page<'a>(components: impl Into<Vec<Comp<'a>>>) -> Comp<'a> {
	layout::Page::new(components, false).into_comp()
}
pub fn page_h<'a>(components: impl Into<Vec<Comp<'a>>>) -> Comp<'a> {
	layout::Page::new(components, true).into_comp()
}
pub fn text<'a>(text: &'a str, size: i32) -> Comp<'a> {
	comp::Text::new(text, size).into_comp()
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
