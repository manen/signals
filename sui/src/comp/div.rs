use crate::core::{Event, Layable};
use crate::{
	comp::{Comp, Compatible},
	Details,
};
use raylib::prelude::RaylibDrawHandle;

pub trait DivComponents {
	type L: Layable;

	fn iter_components(&self) -> impl Iterator<Item = &Self::L>;
}
impl<const N: usize, L: Layable> DivComponents for [L; N] {
	type L = L;

	fn iter_components(&self) -> impl Iterator<Item = &Self::L> {
		self.iter()
	}
}
impl<L: Layable> DivComponents for &[L] {
	type L = L;

	fn iter_components(&self) -> impl Iterator<Item = &Self::L> {
		self.iter()
	}
}
impl<L: Layable> DivComponents for Vec<L> {
	type L = L;

	fn iter_components(&self) -> impl Iterator<Item = &Self::L> {
		self.iter()
	}
}
impl<L: Layable> DivComponents for L {
	type L = L;

	fn iter_components(&self) -> impl Iterator<Item = &Self::L> {
		std::iter::once(self)
	}
}

/// simple page layout, one element after another \
/// just imagine an html div
#[derive(Clone, Debug, Default)]
pub struct Div<D: DivComponents> {
	components: D,
	horizontal: bool,
}
impl<D: DivComponents + Default> Div<D> {
	pub fn empty() -> Self {
		Self::default()
	}
	pub fn horizontal() -> Self {
		Self {
			horizontal: true,
			..Default::default()
		}
	}
}
impl<D: DivComponents> Div<D> {
	pub fn new(horizontal: bool, components: D) -> Self {
		Self {
			components: components,
			horizontal,
		}
	}
}
impl<'a> Div<Vec<Comp<'a>>> {
	pub fn push<C: Compatible<'a>>(&mut self, c: C) {
		self.components.push(c.into_comp());
	}
}
impl<D: DivComponents> Layable for Div<D> {
	fn size(&self) -> (i32, i32) {
		let (mut w, mut h) = (0, 0);

		for comp in self.components.iter_components() {
			let (comp_w, comp_h) = comp.size();

			if !self.horizontal {
				(w, h) = (w.max(comp_w), h + comp_h)
			} else {
				(w, h) = (w + comp_w, h.max(comp_h))
			}
		}

		(w, h)
	}

	fn render(&self, d: &mut RaylibDrawHandle, det: Details, scale: f32) {
		let (self_w, self_h) = self.size();

		let (mut x, mut y) = (det.x, det.y);
		for comp in self.components.iter_components() {
			let (comp_w, comp_h) = comp.size();
			let comp_det = Details {
				x,
				y,
				aw: if !self.horizontal {
					(self_w as f32 * scale) as i32
				} else {
					comp_w
				},
				ah: if self.horizontal {
					(self_h as f32 * scale) as i32
				} else {
					comp_h
				},
			};

			comp.render(d, comp_det, scale);

			if !self.horizontal {
				y += (comp_h as f32 * scale) as i32;
			} else {
				x += (comp_w as f32 * scale) as i32;
			}
		}
	}

	fn pass_event(
		&self,
		event: crate::core::Event,
		det: Details,
		scale: f32,
	) -> Option<crate::core::Event> {
		match event {
			Event::MouseEvent {
				x: mouse_x,
				y: mouse_y,
			} => {
				let (self_w, self_h) = self.size();

				let (mut x, mut y) = (det.x, det.y);
				for comp in self.components.iter_components() {
					let (comp_w, comp_h) = comp.size();
					let comp_det = Details {
						x,
						y,
						aw: if !self.horizontal {
							(self_w as f32 * scale) as i32
						} else {
							comp_w
						},
						ah: if self.horizontal {
							(self_h as f32 * scale) as i32
						} else {
							comp_h
						},
					};

					if comp_det.is_inside(mouse_x, mouse_y) {
						return comp.pass_event(event, comp_det, scale);
					}

					if !self.horizontal {
						y += (comp_h as f32 * scale) as i32;
					} else {
						x += (comp_w as f32 * scale) as i32;
					}
				}
				None
			}
			_ => None,
		}
	}
}
