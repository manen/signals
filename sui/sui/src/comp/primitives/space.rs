use crate::Layable;

#[derive(Copy, Clone, Debug)]
/// Space is literally just some empty space
pub struct Space {
	w: i32,
	h: i32,
}
impl Space {
	pub const fn new(w: i32, h: i32) -> Self {
		Self { w, h }
	}
}
impl Layable for Space {
	fn size(&self) -> (i32, i32) {
		(self.w, self.h)
	}
	fn render(&self, _: &mut crate::Handle, _: crate::Details, _: f32) {}
	fn pass_event(
		&self,
		_: crate::core::Event,
		_: crate::Details,
		_: f32,
	) -> Option<crate::core::ReturnEvent> {
		None
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct MarginValues {
	pub r: i32,
	pub l: i32,
	pub t: i32,
	pub b: i32,
}

#[derive(Clone, Debug)]
pub struct Margin<L: Layable> {
	layable: L,
	values: MarginValues,
}
impl<L: Layable> Margin<L> {
	pub fn new(values: MarginValues, layable: L) -> Self {
		Self { layable, values }
	}

	pub fn vertical(margin_v: i32, layable: L) -> Self {
		Self::new(
			MarginValues {
				r: margin_v,
				l: margin_v,
				t: 0,
				b: 0,
			},
			layable,
		)
	}
	pub fn horizontal(margin_h: i32, layable: L) -> Self {
		Self::new(
			MarginValues {
				r: 0,
				l: 0,
				t: margin_h,
				b: margin_h,
			},
			layable,
		)
	}
	pub fn all(margin: i32, layable: L) -> Self {
		Self::new(
			MarginValues {
				r: margin,
				l: margin,
				t: margin,
				b: margin,
			},
			layable,
		)
	}

	fn l_det(&self, det: crate::Details, scale: f32) -> crate::Details {
		crate::Details {
			x: det.x + ((self.values.l as f32) * scale) as i32,
			y: det.y + ((self.values.t as f32) * scale) as i32,
			aw: det.aw - ((self.values.r as f32 + self.values.l as f32) * scale) as i32,
			ah: det.ah - ((self.values.b as f32 + self.values.t as f32) * scale) as i32,
		}
	}
}
impl<L: Layable> Layable for Margin<L> {
	fn size(&self) -> (i32, i32) {
		let (lw, lh) = self.layable.size();

		(
			lw + self.values.r + self.values.l,
			lh + self.values.t + self.values.b,
		)
	}
	fn render(&self, d: &mut crate::Handle, det: crate::Details, scale: f32) {
		self.layable.render(d, self.l_det(det, scale), scale)
	}
	fn pass_event(
		&self,
		event: crate::core::Event,
		det: crate::Details,
		scale: f32,
	) -> Option<crate::core::ReturnEvent> {
		self.layable
			.pass_event(event, self.l_det(det, scale), scale)
	}
}
