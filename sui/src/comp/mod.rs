mod select_bar;

pub use select_bar::SelectBar;

pub mod clickable;
pub use clickable::Clickable;

pub mod div;
pub use div::Div;

pub mod fit;
pub use fit::*;

pub mod primitives;
pub use primitives::*;

pub mod align;
pub use align::*;

pub mod overlay;
pub use overlay::Overlay;

pub mod debug;
pub use debug::Debug;

pub mod cast_events;
pub use cast_events::CastEvents;

use crate::Layable;

#[derive(Debug, Clone)]
/// this enum contains variants for every base layable (layables that don't have a generic type) \
/// for components with generic types or for anything else really use [Comp::Dynamic] (also [crate::custom])
pub enum Comp<'a> {
	Div(Div<Vec<Comp<'a>>>),
	Text(Text<'a>),
	Space(Space),
	Color(Color),
	Dynamic(crate::core::DynamicLayable<'a>),
}
impl Default for Comp<'static> {
	fn default() -> Self {
		Self::Space(Space::new(0, 0))
	}
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
			Self::Div(a) => a.size(),
			Self::Text(a) => a.size(),
			Self::Space(a) => a.size(),
			Self::Color(a) => a.size(),
			Self::Dynamic(d) => d.size(),
		}
	}
	fn render(&self, d: &mut crate::Handle, det: crate::Details, scale: f32) {
		match self {
			Self::Div(a) => Layable::render(a, d, det, scale),
			Self::Text(a) => a.render(d, det, scale),
			Self::Space(a) => a.render(d, det, scale),
			Self::Color(a) => a.render(d, det, scale),
			Self::Dynamic(dl) => dl.render(d, det, scale),
		}
	}

	fn pass_event(
		&self,
		event: crate::core::Event,
		det: crate::Details,
		scale: f32,
	) -> Option<crate::core::ReturnEvent> {
		match self {
			Self::Div(a) => a.pass_event(event, det, scale),
			Self::Text(a) => a.pass_event(event, det, scale),
			Self::Space(a) => a.pass_event(event, det, scale),
			Self::Color(a) => a.pass_event(event, det, scale),
			Self::Dynamic(dl) => dl.pass_event(event, det, scale),
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
compatible_impl!(Div, Div<Vec<Comp<'a>>>);
compatible_impl!(Text, Text<'a>);
compatible_impl!(Space, Space);
compatible_impl!(Color, Color);
compatible_impl!(Dynamic, crate::DynamicLayable<'a>);
