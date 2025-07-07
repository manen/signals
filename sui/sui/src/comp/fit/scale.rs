use crate::Layable;

#[derive(Clone, Debug)]
pub struct Scale<L: Layable> {
	layable: L,
	scale: f32,
}
impl<L: Layable> Scale<L> {
	pub fn new(layable: L, scale: f32) -> Self {
		Self { layable, scale }
	}
}
impl<L: Layable> Layable for Scale<L> {
	fn size(&self) -> (i32, i32) {
		let (lw, lh) = self.layable.size();
		let (w, h) = (lw as f32 * self.scale, lh as f32 * self.scale);
		(w as i32, h as i32)
	}
	fn render(&self, d: &mut crate::Handle, det: crate::Details, scale: f32) {
		self.layable.render(d, det, scale * self.scale);
	}
	fn pass_event(
		&self,
		event: crate::core::Event,
		det: crate::Details,
		scale: f32,
	) -> Option<crate::core::ReturnEvent> {
		self.layable.pass_event(event, det, scale)
	}
}
