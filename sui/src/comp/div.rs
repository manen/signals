use crate::core::{Event, Layable};
use crate::{
	comp::{Comp, Compatible},
	Details,
};
use raylib::prelude::RaylibDrawHandle;

pub trait DivComponents: Sized {
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
	fill: bool,
}
impl<D: DivComponents + Default> Div<D> {
	pub fn empty() -> Self {
		Self::default()
	}
	pub fn empty_horizontal() -> Self {
		Self {
			horizontal: true,
			..Default::default()
		}
	}
}
impl<D: DivComponents> Div<D> {
	pub fn new(horizontal: bool, fill: bool, components: D) -> Self {
		Self {
			components: components,
			horizontal,
			fill,
		}
	}
	pub fn vertical(components: D) -> Self {
		Self::new(false, false, components)
	}
	pub fn horizontal(components: D) -> Self {
		Self::new(true, false, components)
	}

	pub fn as_horizontal(self) -> Self {
		Self {
			horizontal: true,
			..self
		}
	}
	pub fn as_fill(self) -> Self {
		Self { fill: true, ..self }
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

	fn render(&self, d: &mut crate::Handle, det: Details, scale: f32) {
		let (self_w, self_h) = self.size();

		let (self_w, self_h) = if self.fill {
			(self_w.min(det.aw), self_h.min(det.ah))
		} else {
			(self_w, self_h)
		};

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
	) -> Option<crate::core::ReturnEvent> {
		match event {
			Event::MouseEvent(m_event) => {
				let (mouse_x, mouse_y) = m_event.at();
				let (self_w, self_h) = self.size();

				let (self_w, self_h) = if self.fill {
					(self_w.min(det.aw), self_h.min(det.ah))
				} else {
					(self_w, self_h)
				};

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
			Event::KeyboardEvent(_, _) => {
				for c in self.components.iter_components() {
					if let Some(ret) = c.pass_event(event, det, scale) {
						return Some(ret);
					}
				}
				None
			}
		}
	}
}

impl<L: Layable> FromIterator<L> for Div<Vec<L>> {
	fn from_iter<T: IntoIterator<Item = L>>(iter: T) -> Self {
		let iter = iter.into_iter();
		Div::new(false, false, iter.collect::<Vec<_>>())
	}
}
