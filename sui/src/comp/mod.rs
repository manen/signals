mod select_bar;
pub use select_bar::SelectBar;

pub mod text;
pub use text::Text;

use crate::Layable;

#[derive(Debug, Clone)]
pub enum Comp<'a> {
	Page(crate::layout::Page<'a>),
	Text(Text<'a>),
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
