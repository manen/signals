mod select_bar;
use raylib::prelude::RaylibDraw;
pub use select_bar::SelectBar;

pub mod text;
pub use text::Text;

pub mod clickable;
pub use clickable::Clickable;

use crate::Layable;

#[derive(Debug, Clone)]
pub enum Comp<'a> {
	Page(crate::layout::Page<'a>),
	Text(Text<'a>),
	Clickable(Box<Clickable<Self>>), // it sucks that this has to be a box
}
impl<'a> Comp<'a> {
	pub fn new<C: Compatible<'a>>(c: C) -> Self {
		c.into_comp()
	}
	pub fn take<C: Compatible<'a>>(self) -> Option<C> {
		C::from_comp(self)
	}

	pub fn d<'b>(&self) -> &dyn Layable {
		match self {
			Self::Page(a) => a,
			Self::Text(a) => a,
			Self::Clickable(a) => a.as_ref(),
		}
	}
}

impl<'a> Layable for Comp<'a> {
	fn size(&self) -> (i32, i32) {
		match self {
			Self::Page(a) => a.size(),
			Self::Text(a) => a.size(),
			Self::Clickable(a) => a.size(),
		}
	}
	fn render(&self, d: &mut raylib::prelude::RaylibDrawHandle, det: crate::Details, scale: f32) {
		d.draw_rectangle_lines(
			det.x,
			det.y,
			(det.aw as f32 * scale) as _,
			(det.ah as f32 * scale) as _,
			raylib::color::Color::WHITE,
		);
		match self {
			Self::Page(a) => Layable::render(a, d, det, scale),
			Self::Text(a) => a.render(d, det, scale),
			Self::Clickable(a) => a.render(d, det, scale),
		}
	}

	fn pass_event(&self, event: crate::layout::Event) -> Option<crate::layout::Event> {
		match self {
			Self::Page(a) => a.pass_event(event),
			Self::Text(a) => a.pass_event(event),
			Self::Clickable(a) => a.pass_event(event),
		}
	}
}

pub trait Compatible<'a>: Sized {
	fn from_comp(comp: Comp<'a>) -> Option<Self>;
	fn into_comp(self) -> Comp<'a>;
}
impl<'a> Compatible<'a> for Comp<'a> {
	fn from_comp(comp: Comp<'a>) -> Option<Self> {
		Some(comp)
	}
	fn into_comp(self) -> Comp<'a> {
		self
	}
}

macro_rules! compatible_impl {
	($variant:ident,$ty:ty) => {
		impl<'a> Compatible<'a> for $ty {
			fn from_comp(comp: Comp<'a>) -> Option<Self> {
				match comp {
					Comp::$variant(a) => Some(a),
					_ => None,
				}
			}
			fn into_comp(self) -> Comp<'a> {
				Comp::$variant(self)
			}
		}
	};
}
compatible_impl!(Page, crate::layout::Page<'a>);
compatible_impl!(Text, Text<'a>);

impl<'a> Compatible<'a> for Clickable<Comp<'a>> {
	fn from_comp(comp: Comp<'a>) -> Option<Self> {
		match comp {
			Comp::Clickable(c) => Some(*c),
			_ => None,
		}
	}
	fn into_comp(self) -> Comp<'a> {
		Comp::Clickable(Box::new(self))
	}
}
