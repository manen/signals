use crate::{core::Store, Layable, LayableExt};

use super::{typable::TypableData, FocusCommand, Typable};

/// combines a clickable and a typable to create a component that requests focus when clicked
/// and types if receives keyboardevents and in focus
pub fn textbox(data: Store<TypableData>, text_size: i32) -> impl Layable + Clone + std::fmt::Debug {
	let uid = data.with_borrow(|data| data.uid);

	let typable = Typable::new(data, text_size);
	let clickable = typable.clickable(move |_| FocusCommand::Request(uid));

	clickable.crop()

	// so this works great the component doesn't have focus until we click on it
	// todo:
	// - [x] actually return a dummy event if a typable handles a keyboard event
	// - [x] put every single return event type there is in sui, have some organized way to make a custom return event type
	//         cause right now it's a bit of a mess and these events you have to have a variant for are just scattered wherever they are

	// - [i forgot why this is important] global mouse event fallback that returns a FocusCommand::Drop, i'm thinking in RootContext
	// - [x] also if we're at it we could put a .root_context() in LayableExt
	// - [x] and we could make an element that turns ReturnEvent<(any type of sui specific return type)> into a ReturnEvent<SignalsEvent(or any event that implements From<X> where X is is every sui specific return type)>
	// - [x] and then we can drop the generics from this function
}
