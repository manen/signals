use crate::{core::FeaturedReturn, Layable};

/// CastEvents casts compatible events into a type that can hold either of
/// said compatible events
///
/// this is required in many cases but [crate::RootContext] will take care of it
#[derive(Clone, Debug)]
pub struct CastEvents<E: FeaturedReturn, L: Layable> {
	layable: L,
	_e: std::marker::PhantomData<E>,
}
impl<E: FeaturedReturn, L: Layable> CastEvents<E, L> {
	pub fn new(layable: L) -> Self {
		Self {
			layable,
			_e: Default::default(),
		}
	}
}
impl<E: FeaturedReturn, L: Layable> Layable for CastEvents<E, L> {
	fn size(&self) -> (i32, i32) {
		self.layable.size()
	}
	fn render(&self, d: &mut crate::Handle, det: crate::Details, scale: f32) {
		self.layable.render(d, det, scale);
	}
	fn pass_event(
		&self,
		event: crate::core::Event,
		det: crate::Details,
		scale: f32,
	) -> Option<crate::core::ReturnEvent> {
		self.layable
			.pass_event(event, det, scale)
			.map(|event| E::cast_event(event))
	}
}
