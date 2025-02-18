use raylib::prelude::RaylibDraw;
use sui::{comp, Comp, Layable, LayableExt};

fn should_glow(i: i32, time: f64) -> bool {
	let time = time * 6.0;
	let n = time + i as f64 * -0.3;
	let fx = n.sin();

	fx > 0.8
}

/// does not set self.size(), use a FixedSize
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TitleLetter {
	i: i32,
	c: char,
	size: i32,

	self_s: (i32, i32),
}
impl TitleLetter {
	fn new(i: i32, c: char, size: i32) -> Self {
		let mut buffer = [0; 4]; // A char can be at most 4 bytes in UTF-8
		let text: &str = c.encode_utf8(&mut buffer);
		let text = comp::Text::new(text, size);

		let self_s = text.size();

		Self { i, c, size, self_s }
	}
}
impl Layable for TitleLetter {
	fn size(&self) -> (i32, i32) {
		self.self_s
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		let mut buffer = [0; 4]; // A char can be at most 4 bytes in UTF-8
		let text: &str = self.c.encode_utf8(&mut buffer);

		let color = if should_glow(self.i, d.get_time()) {
			crate::gfx::WIRE_ON
		} else {
			comp::text::DEFAULT_COLOR
		};
		d.draw_text(text, det.x, det.y, self.size, color);
	}
	fn pass_event(
		&self,
		_event: sui::core::Event,
		_det: sui::Details,
		_scale: f32,
	) -> Option<sui::core::ReturnEvent> {
		None
	}
}

pub fn title() -> impl Layable + Clone + std::fmt::Debug {
	let title = "signals";
	let font_size = 32;

	title_with(title, font_size)
}

pub fn title_with(title: &str, font_size: i32) -> impl Layable + Clone + std::fmt::Debug {
	let would_be = comp::Text::new(title, font_size);

	let letters = title
		.chars()
		.enumerate()
		.map(|(i, c)| TitleLetter::new(i as i32, c, font_size))
		.map(|c| c.margin(1))
		.collect::<comp::Div<_>>()
		.as_horizontal();

	letters
}
