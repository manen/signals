use crate::{game::Game, ui::ingame::WorldPreview, world::World};
use sui::{Compatible, Debuggable};

pub const SWITCH_CLICKED: &str = "worlds_bar_worlds_switch_clicked";
pub const FOREIGN_CLICKED: &str = "worlds_bar_worlds_place_clicked";
pub const PLUS_CLICKED: &str = "worlds_bar_worlds_plus_clicked";

pub fn worlds_bar(game: &Game, height: i32) -> sui::Comp {
	// sui::page(vec![sui::custom(sui::comp::ScaleToFit::fix_h(
	// 	WorldPreview::new(game.main.as_ref()),
	// 	200,
	// ))])

	let previews = game
		.worlds()
		.enumerate()
		.map(|(i, w)| worlds_bar_world(height, i, w))
		.chain(std::iter::once(sui::custom(
			sui::comp::FixedSize::fix_both(
				sui::comp::Clickable::new(
					sui::comp::Centered::new(sui::comp::Text::new("+", 50)),
					PLUS_CLICKED,
					0,
				),
				height,
			),
		)));

	sui::custom(sui::comp::Clickable::new_fallback(
		sui::comp::Div::new(previews.collect::<Vec<_>>(), true),
		"faszopm kivan mar",
		6,
	))
	// sui::page_h(previews.collect::<Vec<_>>())
}

fn worlds_bar_world(height: i32, i: usize, w: &World) -> sui::Comp {
	sui::custom(sui::comp::Overlay::new(
		sui::comp::ScaleToFit::fix_h(WorldPreview::new(w), height),
		sui::comp::Centered::new(sui::Div::new(
			vec![
				sui::custom(sui::comp::Clickable::new(
					// sui::comp::Centered::new(
					sui::comp::Text::new("switch here", 14), // )
					SWITCH_CLICKED,
					i as i32,
				)),
				sui::comp::Space::new(0, 20).into_comp(),
				sui::custom(sui::comp::Clickable::new(
					sui::comp::Centered::new(sui::comp::Text::new("place", 14)),
					FOREIGN_CLICKED,
					i as i32,
				)),
			],
			false,
		)),
	))
}
