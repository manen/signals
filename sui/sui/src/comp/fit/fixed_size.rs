use crate::Layable;

#[derive(Copy, Clone, Debug)]
/// is the width or the height going to be fixed
pub enum FitOpt {
	Width(i32),
	Height(i32),
	Both((i32, i32)),
}

#[derive(Clone, Debug)]
/// makes either the width or the height fixed. will still render the component inside at 0,0
pub struct FixedSize<L: Layable> {
	layable: L,
	fit_opt: FitOpt,
}
impl<L: Layable> FixedSize<L> {
	pub fn new(layable: L, fit_opt: FitOpt) -> Self {
		Self { layable, fit_opt }
	}
	pub fn fix_w(width: i32, layable: L) -> Self {
		Self::new(layable, FitOpt::Width(width))
	}
	pub fn fix_h(height: i32, layable: L) -> Self {
		Self::new(layable, FitOpt::Height(height))
	}
	pub fn fix_size(size: (i32, i32), layable: L) -> Self {
		Self::new(layable, FitOpt::Both(size))
	}
	pub fn fix_both(both: i32, layable: L) -> Self {
		Self::fix_size((both, both), layable)
	}

	fn l_det(&self, det: crate::Details) -> crate::Details {
		let (w, h) = self.size();
		crate::Details {
			x: det.x,
			y: det.y,
			aw: w,
			ah: h,
		}
	}
}
impl<L: Layable> Layable for FixedSize<L> {
	fn size(&self) -> (i32, i32) {
		match self.fit_opt {
			FitOpt::Width(w) => (w, self.layable.size().0),
			FitOpt::Height(h) => (self.layable.size().1, h),
			FitOpt::Both(s) => s,
		}
	}
	fn render(&self, d: &mut crate::Handle, det: crate::Details, scale: f32) {
		self.layable.render(d, self.l_det(det), scale)
	}
	fn pass_event(
		&self,
		event: crate::core::Event,
		det: crate::Details,
		scale: f32,
	) -> Option<crate::core::ReturnEvent> {
		self.layable.pass_event(event, self.l_det(det), scale)
	}
}
