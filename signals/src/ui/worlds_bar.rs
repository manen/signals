use crate::{game::Game, ui::ingame::WorldPreview};

pub fn worlds_bar(game: &Game) -> sui::Comp {
	sui::page(vec![sui::custom(WorldPreview::new(game.main.as_ref()))])
}
