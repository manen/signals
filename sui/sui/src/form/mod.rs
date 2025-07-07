pub mod typable;
use std::rc::Rc;

pub use typable::Typable;

pub mod textbox;
pub use textbox::textbox;
use typable::TypableData;

use crate::{core::Store, Layable};

// i don't know if this is the appropriate place for the focus implementation

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
