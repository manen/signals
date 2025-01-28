pub mod ingame;

pub mod worlds_bar;
use fit::scrollable::ScrollableState;

use sui::comp::div::DivComponents;

use crate::{
	game::{IngameWorld, IngameWorldType, WorldId},
	processor,
};
use sui::{comp::*, core::Store, LayableExt};

#[derive(Clone, Debug)]
pub enum SignalsEvent {
	DialogCommand(sui::dialog::Command),
	DialogFallback,
	NewWorld,
	SwitchToWorld(WorldId),
	PlaceWorld(WorldId),
	WorldsBarFallback,
}

fn spawn_dialog() -> sui::comp::Comp<'static> {
	let comp = Text::new("summon dialog", 24).clickable(|(x, y)| {
		let dialog_content =
			Div::new(
				false,
				[
					sui::custom(Text::new("this is a dialog!!! yippie", 16).centered()),
					sui::custom(Space::new(30, 30)),
					sui::custom(Text::new("close", 12).clickable(move |_| {
						SignalsEvent::DialogCommand(sui::dialog::Command::Close)
					})),
				],
			);
		let dialog_content = sui::custom(dialog_content);

		SignalsEvent::DialogCommand(sui::dialog::Command::Open(sui::dialog::Instance {
			comp: dialog_content,
			at: (x, y),
			scale: 1.0,
		}))
	});

	sui::custom(comp)
}

pub fn game_debug_ui(
	game: &crate::Game,
	scroll_state: Store<ScrollableState>,
) -> sui::comp::Comp<'static> {
	let ingameworld_dbg = ingameworld_dbg_ui(0, &game.moves);

	let page = Div::new(
		false,
		[
			spawn_dialog(),
			ingameworld_dbg,
			sui::custom(sui::comp::Text::new("this is centered!!!", 13).centered()),
			inst_comp(game, game.main_id),
		],
	);

	sui::custom(page.scrollable(scroll_state).fix_wh_square(300))
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
		.map(|(i, child)| ingameworld_dbg_ui(i, child).margin(2));
	let children_div = children.collect::<Vec<_>>().to_div();

	sui::custom(
		[sui::custom(line.into_comp()), sui::custom(children_div)]
			.to_div()
			.margin(2),
	)
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
		Err(err) => sui::text(format!("{err:#?}"), 16),
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
