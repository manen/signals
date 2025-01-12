pub mod ingame;

pub mod worlds_bar;
use fit::scrollable::ScrollableState;
pub use worlds_bar::worlds_bar;

use crate::{game::IngameWorld, processor};
use sui::{comp::*, core::Store};

pub fn game_debug_ui(
	game: &crate::Game,
	scroll_state: Store<ScrollableState>,
) -> sui::comp::Comp<'static> {
	let ingameworld_dbg = ingameworld_dbg_ui(0, &game.moves);

	let page = Div::new(
		false,
		[
			ingameworld_dbg,
			sui::custom(sui::comp::Centered::new(sui::comp::Text::new(
				"this is centered!!!",
				13,
			))),
		],
	);

	sui::custom(FixedSize::fix_both(
		300,
		Scrollable::new(scroll_state, fit::scrollable::ScrollableMode::Both, page),
	))
}

fn ingameworld_dbg_ui(i: usize, moves: &IngameWorld) -> sui::comp::Comp<'static> {
	let line = Text::new(format!("inst {i}: {:?}", moves.world_id), 12);

	let children = moves
		.children
		.iter()
		.enumerate()
		.map(|(i, child)| ingameworld_dbg_ui(i, child));
	let children_div = Div::new(
		true,
		[
			Space::new(10, 0).into_comp(),
			sui::page(children.collect::<Vec<_>>()),
		],
	);

	sui::custom(Div::new(
		false,
		[
			line.into_comp(),
			Space::new(0, 5).into_comp(),
			sui::custom(children_div),
		],
	))
}

pub fn inst_comp(game: &crate::Game, world_id: Option<usize>) -> sui::Comp<'static> {
	let instructions = processor::world_to_instructions(game, world_id);

	let lines = instructions
		.into_iter()
		.map(|inst| Text::new(format!("{inst:?}"), 16));
	let lines = lines.collect::<Vec<_>>();

	sui::custom(Div::new(false, lines))
}
