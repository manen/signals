pub mod ingame;

pub mod worlds_bar;
use fit::scrollable::ScrollableState;
pub use worlds_bar::worlds_bar;

use sui::{comp::*, core::Store, Debuggable};

pub fn game_debug_ui(
	game: &crate::Game,
	scroll_state: Store<ScrollableState>,
) -> sui::comp::Comp<'static> {
	let lines = game
		.moves
		.children
		.iter()
		.enumerate()
		.map(|(i, ingameworld)| format!("inst {i}: {:?}", ingameworld.world_id))
		.map(|s| {
			[
				sui::comp::Text::new(s, 12).into_comp(),
				sui::comp::Space::new(0, 5).into_comp(), // <- this is kinda just for testing i want to make a margin component later
			]
		})
		.flatten();

	let content = lines
		.chain(std::iter::once(sui::custom(
			Text::new(format!("{:#?}", game.moves), 16).debug(),
		)))
		.chain(std::iter::once(sui::custom(sui::comp::Centered::new(
			sui::comp::Text::new("this is centered!!!", 13),
		))));
	let page = Div::new(false, content.collect::<Vec<_>>());

	sui::custom(FixedSize::fix_both(
		300,
		Scrollable::new(scroll_state, fit::scrollable::ScrollableMode::Both, page),
	))
}
