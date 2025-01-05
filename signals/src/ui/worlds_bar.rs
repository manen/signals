use crate::{game::Game, ui::ingame::WorldPreview, world::World};
use sui::{comp::*, Compatible};

pub const SWITCH_CLICKED: &str = "worlds_bar_worlds_switch_clicked";
pub const FOREIGN_CLICKED: &str = "worlds_bar_worlds_place_clicked";
pub const PLUS_CLICKED: &str = "worlds_bar_worlds_plus_clicked";

pub fn worlds_bar(game: &Game, height: i32) -> sui::Comp {
	let previews = std::iter::once(worlds_bar_main(height, game.world_main()))
		.chain(
			game.worlds()
				.enumerate()
				.map(|(i, w)| worlds_bar_world(height, i, w)),
		)
		.chain(std::iter::once(sui::custom(FixedSize::fix_both(
			height,
			Clickable::new(PLUS_CLICKED, 0, Centered::new(Text::new("+", 50))),
		))));

	sui::custom(Clickable::new_fallback(
		"faszopm kivan mar",
		6,
		Div::new(true, previews.collect::<Vec<_>>()),
	))
}

/// i is 0 on main, i + 1 on Some(i)
fn worlds_bar_world(height: i32, i: usize, w: &World) -> sui::Comp {
	sui::custom(Overlay::new(
		ScaleToFit::fix_h(height, WorldPreview::new(w)),
		FixedSize::fix_both(
			height,
			Centered::new(sui::Div::new(
				false,
				[
					sui::custom(Clickable::new(
						FOREIGN_CLICKED,
						i as i32 + 1,
						Centered::new(Text::new("place", 14)),
					)),
					Space::new(0, 20).into_comp(),
					sui::custom(Clickable::new(
						SWITCH_CLICKED,
						i as i32 + 1,
						Text::new("switch here", 14),
					)),
				],
			)),
		),
	))
}

fn worlds_bar_main(height: i32, w: &World) -> sui::Comp {
	sui::custom(Clickable::new(
		SWITCH_CLICKED,
		0,
		Overlay::new(
			ScaleToFit::fix_h(height, WorldPreview::new(w)),
			Centered::new(FixedSize::fix_both(
				height,
				Centered::new(Text::new("switch here", 14)),
			)),
		),
	))
}
