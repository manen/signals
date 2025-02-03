use crate::{core::Store, Layable, LayableExt};

use super::{typable::TypableData, FocusCommand, Typable};

/// combines a clickable and a typable to create a component that requests focus when clicked
/// and types if receives keyboardevents and in focus
pub fn textbox(data: Store<TypableData>, text_size: i32) -> impl Layable + Clone + std::fmt::Debug {
	let uid = data.with_borrow(|data| data.uid);

	let typable = Typable::new(data, text_size);
	let clickable = typable.clickable(move |_| FocusCommand::Request(uid));

	clickable.crop()
}
