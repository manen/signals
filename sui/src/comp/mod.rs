mod select_bar;
use std::fmt::Debug;

use raylib::prelude::RaylibDraw;
pub use select_bar::SelectBar;

pub mod text;
pub use text::Text;

pub mod clickable;
pub use clickable::Clickable;

pub mod div_box;
pub use div_box::Box;

pub mod fit;
pub use fit::{Centered, FixedSize, ScaleToFit};

use crate::Layable;

#[derive(Debug, Clone)]
/// this enum contains variants for every base layable (layables that don't have a generic type) \
/// for components with generic types or for anything else really use [Comp::Dynamic] (also [crate::custom])
pub enum Comp<'a> {
	Page(Box<'a>),
	Text(Text<'a>),
	Dynamic(crate::core::DynamicLayable<'a>),
}
impl<'a> Comp<'a> {
	pub fn new<C: Compatible<'a>>(c: C) -> Self {
		c.into_comp()
	}
	pub fn take<C: Compatible<'a>>(self) -> Option<C> {
		C::from_comp(self)
	}
}

impl<'a> Layable for Comp<'a> {
	fn size(&self) -> (i32, i32) {
		match self {
			Self::Page(a) => a.size(),
			Self::Text(a) => a.size(),
			Self::Dynamic(d) => d.size(),
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
			Self::Dynamic(dl) => dl.render(d, det, scale),
		}
	}

	fn pass_event(&self, event: crate::core::Event) -> Option<crate::core::Event> {
		match self {
			Self::Page(a) => a.pass_event(event),
			Self::Text(a) => a.pass_event(event),
			Self::Dynamic(dl) => dl.pass_event(event),
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
compatible_impl!(Page, Box<'a>);
compatible_impl!(Text, Text<'a>);

compatible_impl!(Dynamic, crate::DynamicLayable<'a>);
