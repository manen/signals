use crate::Layable;

#[derive(Copy, Clone, Debug)]
/// is the width or the height going to be fixed
pub enum FitOpt {
	Width(i32),
	Height(i32),
}

#[derive(Clone, Debug)]
/// ScaleToFit renders layable, scaling it to fit `self.fit_opt`
pub struct ScaleToFit<L: Layable> {
	layable: L,
	fit_opt: FitOpt,
}
impl<L: Layable> ScaleToFit<L> {
	pub fn new(layable: L, fit_opt: FitOpt) -> Self {
		Self { layable, fit_opt }
	}
	pub fn fix_w(layable: L, width: i32) -> Self {
		Self::new(layable, FitOpt::Width(width))
	}
	pub fn fix_h(layable: L, height: i32) -> Self {
		Self::new(layable, FitOpt::Height(height))
	}

	/// size is Option<self.layable.size()>
	pub fn scale(&self, size: Option<(i32, i32)>) -> f32 {
		let (l_w, l_h) = size.unwrap_or_else(|| self.layable.size());

		match self.fit_opt {
			FitOpt::Width(w) => w as f32 / l_w as f32,
			FitOpt::Height(h) => h as f32 / l_h as f32,
		}
	}
}
impl<L: Layable> Layable for ScaleToFit<L> {
	fn size(&self) -> (i32, i32) {
		let (l_w, l_h) = self.layable.size();
		let scale = self.scale(Some((l_w, l_h)));

		((l_w as f32 * scale) as i32, (l_h as f32 * scale) as i32)
	}
	fn render(&self, d: &mut raylib::prelude::RaylibDrawHandle, det: crate::Details, scale: f32) {
		self.layable.render(d, det, scale * self.scale(None));
	}
	fn pass_event(
		&self,
		event: crate::core::Event,
		det: crate::Details,
		scale: f32,
	) -> Option<crate::core::Event> {
		self.layable
			.pass_event(event, det, scale * self.scale(None))
	}
}

#[cfg(test)]
mod scaletofit_tests {
	use super::*;

	#[test]
	fn test_scaling() {
		struct Dummy;
		impl Layable for Dummy {
			fn size(&self) -> (i32, i32) {
				(100, 200)
			}
			fn render(&self, _: &mut raylib::prelude::RaylibDrawHandle, _: crate::Details, _: f32) {
			}
			fn pass_event(
				&self,
				event: crate::core::Event,
				_: crate::Details,
				_: f32,
			) -> Option<crate::core::Event> {
				Some(event)
			}
		}
		{
			let stf = ScaleToFit::fix_w(Dummy, 50);
			assert_eq!(stf.size(), (50, 100));
		}
		{
			let stf = ScaleToFit::fix_h(Dummy, 400);
			assert_eq!(stf.size(), (200, 400));
		}
	}
}
