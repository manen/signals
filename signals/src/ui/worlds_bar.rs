use crate::{game::Game, ui::ingame::WorldPreview};

pub fn worlds_bar(game: &Game, height: i32) -> sui::Comp {
	// sui::page(vec![sui::custom(sui::comp::ScaleToFit::fix_h(
	// 	WorldPreview::new(game.main.as_ref()),
	// 	200,
	// ))])

	let previews = game
		.worlds()
		.enumerate()
		.map(|(i, a)| {
			println!("{i}");
			a
		})
		.map(|w| sui::custom(sui::comp::ScaleToFit::fix_h(WorldPreview::new(w), height)));

	sui::page_h(previews.collect::<Vec<_>>())
}
