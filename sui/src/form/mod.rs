pub mod typable;
pub use typable::Typable;

use crate::core::Store;

pub type FocusHandler = Store<UniqueId>;
pub fn focus_handler() -> FocusHandler {
	FocusHandler::new(UniqueId::null())
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum FocusCommand {
	Request(UniqueId),
	Drop,
}
impl FocusCommand {
	pub fn apply(&self, fh: &mut FocusHandler) {
		match self {
			&FocusCommand::Request(uid) => fh.set(uid),
			&FocusCommand::Drop => fh.set(UniqueId::null()),
		};
	}
}

// use rand::RngCore;
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct UniqueId(u32);
impl UniqueId {
	pub const fn null() -> Self {
		Self(0)
	}
	pub fn new() -> Self {
		use rand::RngCore;
		use rand_core::SeedableRng;
		use rand_pcg::Pcg64Mcg;

		let mut rng = Pcg64Mcg::from_rng(&mut rand::rng());
		Self(rng.next_u32())
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormEvent {
	/// sent by the input to the form to let the form know the input exists
	Register(UniqueId),
	/// requests an input to return the collected value
	CollectFrom(UniqueId),
}

pub enum InputValue {
	Text(String),
}
