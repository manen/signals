pub mod ingame;

pub mod worlds_bar;
pub use worlds_bar::worlds_bar;

use sui::comp::Compatible;

pub fn game_debug_ui(game: &crate::Game) -> sui::comp::Comp {
	let lines = game
		.moves
		.children
		.iter()
		.enumerate()
		.map(|(i, ingameworld)| format!("inst {i}: {:?}", ingameworld.world_id))
		.map(|s| sui::comp::Text::new(s, 12).into_comp());

	let content = lines
		.chain(std::iter::once(sui::text(format!("{:#?}", game.moves), 16)))
		.chain(std::iter::once(sui::custom(sui::comp::Centered::new(
			sui::comp::Text::new("this is centered!!!", 13),
		))));
	let page = sui::page(content.collect::<Vec<_>>());

	page.into_comp()
}
