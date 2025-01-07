use crate::{game::Game, ui::ingame::WorldPreview, world::World};
use fit::scrollable::{self, ScrollableState};
use raylib::prelude::RaylibDrawHandle;
use sui::{comp::*, core::Store, tex::Texture, Compatible};

pub const SWITCH_CLICKED: &str = "worlds_bar_worlds_switch_clicked";
pub const FOREIGN_CLICKED: &str = "worlds_bar_worlds_place_clicked";
pub const PLUS_CLICKED: &str = "worlds_bar_worlds_plus_clicked";

pub fn worlds_bar(
	d: &mut RaylibDrawHandle,
	game: &Game,
	height: i32,
	scroll_state: Store<ScrollableState>,
) -> sui::Comp<'static> {
	println!("recreating world_bar");
	let previews = std::iter::once(worlds_bar_main(d, height, game.world_main()))
		.chain(
			game.worlds()
				.enumerate()
				.map(|(i, w)| worlds_bar_world(d, height, i, w)),
		)
		.chain(std::iter::once(sui::custom(FixedSize::fix_both(
			height,
			Clickable::new(PLUS_CLICKED, 0, Centered::new(Text::new("+", 50))),
		))));
	let previews = previews.collect::<Vec<_>>();

	sui::custom(FixedSize::fix_size(
		(
			d.get_render_width(),
			height + scrollable::SCROLLBAR_WIDTH as i32,
		),
		Scrollable::new(
			scroll_state,
			fit::scrollable::ScrollableMode::Vertical,
			Clickable::new_fallback("faszopm kivan mar", 6, Div::new(true, previews)).debug(),
		)
		.debug(),
	))
}

/// i is 0 on main, i + 1 on Some(i)
fn worlds_bar_world(
	d: &mut RaylibDrawHandle,
	height: i32,
	i: usize,
	w: &World,
) -> sui::Comp<'static> {
	let world_preview = ScaleToFit::fix_h(height, WorldPreview::new(w));

	sui::custom(Overlay::new(
		Texture::from_layable(d, &world_preview),
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

fn worlds_bar_main(d: &mut RaylibDrawHandle, height: i32, w: &World) -> sui::Comp<'static> {
	let world_preview = ScaleToFit::fix_h(height, WorldPreview::new(w));

	sui::custom(Clickable::new(
		SWITCH_CLICKED,
		0,
		Overlay::new(
			Texture::from_layable(d, &world_preview),
			Centered::new(FixedSize::fix_both(
				height,
				Centered::new(Text::new("switch here", 14)),
			)),
		),
	))
}
