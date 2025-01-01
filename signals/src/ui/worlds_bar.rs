use crate::{game::Game, ui::ingame::WorldPreview};

pub fn worlds_bar(game: &Game, height: i32) -> sui::Comp {
	// sui::page(vec![sui::custom(sui::comp::ScaleToFit::fix_h(
	// 	WorldPreview::new(game.main.as_ref()),
	// 	200,
	// ))])

	let previews = game
		.worlds()
		.map(|w| sui::custom(sui::comp::ScaleToFit::fix_h(WorldPreview::new(w), height)))
		.chain(std::iter::once(sui::custom(
			sui::comp::FixedSize::fix_both(
				sui::comp::Centered::new(sui::comp::Text::new("+", 50)),
				height,
			),
		)));

	sui::custom(sui::comp::FixedSize::fix_h(
		sui::comp::Box::new(previews.collect::<Vec<_>>(), true),
		height,
	))
	// sui::page_h(previews.collect::<Vec<_>>())
}
