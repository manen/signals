pub mod ingame;

pub mod worlds_bar;
use fit::scrollable::ScrollableState;
pub use worlds_bar::worlds_bar;

use crate::{
	game::{IngameWorld, IngameWorldType, WorldId},
	processor,
};
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
			inst_comp(game, game.main_id),
		],
	);

	sui::custom(FixedSize::fix_both(
		300,
		Scrollable::new(scroll_state, fit::scrollable::ScrollableMode::Both, page),
	))
}

fn ingameworld_dbg_ui(i: usize, moves: &IngameWorld) -> sui::comp::Comp<'static> {
	let typ = match moves.typ {
		IngameWorldType::Simulated { .. } => "",
		IngameWorldType::Processor { .. } => " proc",
	};
	let line = Text::new(format!("inst {i}:{} {:?}", typ, moves.world_id), 12);

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

pub fn inst_comp(game: &crate::Game, world_id: WorldId) -> sui::Comp<'static> {
	let insts = processor::world_to_instructions(game, world_id);

	let insts = match insts {
		Ok(instructions) => {
			let lines = instructions
				.into_iter()
				.map(|inst| Text::new(format!("{inst:?}"), 16));
			let lines = lines.collect::<Vec<_>>();

			sui::custom(Div::new(false, lines))
		}
		Err(err) => sui::text(format!("{err}"), 16),
	};

	let eq = game
		.worlds
		.at(world_id)
		.map(|a| {
			a.outputs()
				.filter(|(id, _)| *id == 0)
				.map(|(_, coords)| {
					processor::world_to_instructions::world_block_to_eq(game, world_id, coords)
				})
				.next()
		})
		.flatten();

	let eq = match eq {
		Some(Ok(eq)) => sui::text(format!("{eq:#?}"), 16),
		_ => sui::text(format!("{eq:#?}"), 16),
	};
	let eq = sui::page([sui::text("equation: ", 18), eq]);

	sui::custom(Div::new(false, [insts, eq]))
}
