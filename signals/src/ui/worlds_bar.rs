use crate::{game::Game, ui::ingame::WorldPreview, world::World};
use fit::scrollable::{self, ScrollableState};
use raylib::prelude::RaylibDrawHandle;
use sui::{
	comp::*,
	core::{Cached, Store},
	tex::Texture,
	Compatible, Details, LayableExt, RootContext,
};

pub const SWITCH_CLICKED: &str = "worlds_bar_worlds_switch_clicked";
pub const FOREIGN_CLICKED: &str = "worlds_bar_worlds_place_clicked";
pub const PLUS_CLICKED: &str = "worlds_bar_worlds_plus_clicked";

#[derive(Default)]
pub struct WorldsBar {
	comp: Cached<Comp<'static>>,
	scroll_state: Store<ScrollableState>,
}
impl WorldsBar {
	pub fn new(d: &mut RaylibDrawHandle, game: &Game, height: i32) -> Self {
		let mut s = Self {
			comp: Default::default(),
			scroll_state: Default::default(),
		};
		s.update(d, game, height);
		s
	}

	pub fn update(&mut self, d: &mut RaylibDrawHandle, game: &Game, height: i32) -> &Comp<'static> {
		self.comp.update_with_unchecked(height, d, |height, d| {
			worlds_bar(d, game, height, self.scroll_state.clone())
		})
	}
	pub fn clear_cache(&mut self) {
		self.comp = Default::default();
	}

	pub fn root_context(&self, det: Details, scale: f32) -> RootContext<Comp<'static>> {
		match self.comp.borrow() {
			Some(comp) => RootContext::new(comp, det, scale),
			None => panic!("attempted to get RootContext from uninitialized WorldsBar"),
		}
	}
}
// eventually abstract all this into a CustomComp trait if i feel like it

fn worlds_bar(
	d: &mut RaylibDrawHandle,
	game: &Game,
	height: i32,
	scroll_state: Store<ScrollableState>,
) -> sui::Comp<'static> {
	println!("recreating world_bar");
	let previews = game
		.worlds()
		.enumerate()
		.map(|(i, w)| worlds_bar_world(d, height, i, w))
		.chain(std::iter::once(sui::custom(
			Text::new("+", 50)
				.centered()
				.clickable(PLUS_CLICKED, 0)
				.fix_wh_square(height),
		)));
	let previews = previews.collect::<Vec<_>>();

	let elem = Div::new(true, previews)
		.scrollable_horiz(scroll_state)
		.fix_wh(
			d.get_render_width(),
			height + scrollable::SCROLLBAR_WIDTH as i32,
		)
		.clickable_fallback("faszopm kivan mar", 6);

	sui::custom(elem)
}

fn worlds_bar_world(
	d: &mut RaylibDrawHandle,
	height: i32,
	i: usize,
	w: &World,
) -> sui::Comp<'static> {
	let world_preview = ScaleToFit::fix_h(height, WorldPreview::new(w));

	let place = Text::new("place", 14)
		.centered()
		.clickable(FOREIGN_CLICKED, i as i32);
	let switch = Text::new("switch here", 14).centered();

	let clickables = Div::new(
		false,
		[
			sui::custom(switch),
			Space::new(0, 20).into_comp(),
			sui::custom(place),
		],
	);

	let elem = clickables
		.centered()
		.fix_wh_square(height)
		.with_background(Texture::from_layable(d, &world_preview))
		.clickable_fallback(SWITCH_CLICKED, i as i32);

	sui::custom(elem)
}
