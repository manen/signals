mod foreign_clump;

use foreign_clump::FindClump;

use crate::{
	game::{Game, WorldId},
	world::{Block, Direction},
};

pub const TOOLS: &[(&str, Tool)] = &[
	("place wire", Tool::PlaceWire { start: None }),
	("place switch", Tool::Place(Block::Switch(false))),
	("place junction", Tool::Place(Block::Junction)),
	("place router", Tool::Place(Block::Router)),
	("place not", Tool::Place(Block::Not(false))),
	("place input", Tool::PlaceInput),
	("place output", Tool::PlaceOutput),
	("remove", Tool::Place(Block::Nothing)),
	("rotate", Tool::Rotate),
	("copy", Tool::Copy),
	("interact", Tool::Interact),
];

macro_rules! main_or_return {
	($game:expr) => {{
		match $game.worlds.at($game.main_id) {
			Some(a) => a,
			None => return,
		}
	}};
	(mut $game:expr) => {{
		match $game.worlds.at_mut($game.main_id) {
			Some(a) => a,
			None => return,
		}
	}};
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Tool {
	PlaceWire {
		start: Option<(i32, i32)>,
	},
	Place(Block),
	PlaceInput,
	PlaceOutput,
	PlaceForeign(WorldId), // world id

	Rotate,
	Copy,

	#[default]
	Interact,
}
impl Tool {
	// pub fn rotate(self) -> Self {
	// 	match self {
	// 		Self::PlaceWire { .. } => Self::Place(Block::Nothing),
	// 		Self::Place(Block::Nothing) => Self::Place(Block::Switch(false)),
	// 		Self::Place(Block::Switch(_)) => Self::Place(Block::Not(false)),
	// 		Self::Place(Block::Not(_)) => Self::Rotate,
	// 		Self::Rotate => Self::Interact,
	// 		Self::Interact => Self::PlaceWire { start: None },
	// 		_ => Self::PlaceWire { start: None },
	// 	}
	// }

	pub fn down(&mut self, x: i32, y: i32, game: &mut Game) {
		match self {
			Self::Place(block) => {
				let main = main_or_return!(game);
				match main.at(x, y) {
					Some(Block::Input(_) | Block::Output(_)) => {
						main_or_return!(mut game).io_blocks_fix();
					}
					Some(Block::Foreign(_, _, _)) => match game.regenerate_moves(game.main_id) {
						Ok(a) => a,
						Err(err) => eprintln!(
							"failed to regenerate moves after replacing a foreign block:\n{err}"
						),
					},
					_ => (),
				}
				let main = main_or_return!(mut game);
				let ptr = main.mut_at(x, y);
				*ptr = *block;
			}
			_ => {}
		}
	}
	pub fn pressed(&mut self, x: i32, y: i32, game: &mut Game) {
		let main = main_or_return!(mut game);
		match self {
			Self::Rotate => main.map_at(x, y, |i| match i {
				Block::Wire(dir) => Block::Wire(dir.rotate_r()),
				_ => i,
			}),
			Self::Copy => {
				*self = match main.at(x, y).copied().unwrap_or_default() {
					Block::Input(_) => Tool::PlaceInput,
					Block::Output(_) => Tool::PlaceOutput,
					Block::Foreign(wid, _, _) => Tool::PlaceForeign(wid),

					block => Tool::Place(block),
				}
			}
			Self::PlaceWire { start } if *start == None => *start = Some((x, y)),
			Self::Interact => main.mut_at(x, y).interact(),
			Self::PlaceInput => {
				*main.mut_at(x, y) = Block::Input(main.inputs_count());
				main.io_blocks_fix();
				// TODO if io_blocks_inputs_len() worked properly we wouldn't need to fix io blocks
				// immediately afterwards
			}
			Self::PlaceOutput => {
				*main.mut_at(x, y) = Block::Output(main.outputs_count());
				main.io_blocks_fix();
				// TODO if io_blocks_outputs_len() worked properly we wouldn't need to fix io blocks
				// immediately afterwards
			}
			Self::PlaceForeign(wid) => {
				let main = match game.worlds.at_mut(game.main_id) {
					Some(a) => a,
					None => return,
				};

				let mut clump = main
					.find_clump((x, y))
					.foreign_data()
					.filter_map(|(this_wid, inst_id, id)| {
						if this_wid == *wid {
							Some((inst_id, id))
						} else {
							None
						}
					})
					.collect::<Vec<_>>();
				clump.sort_by(|(a_inst_id, a_id), (b_inst_id, b_id)| {
					(a_inst_id * 1000 + a_id).cmp(&(b_inst_id * 1000 + b_id))
				});

				// println!("{clump:#?}");

				let max_id = match game.programs.get(wid) {
					Some(&(_, a, b)) => a.min(b),
					None => return,
				};

				let (new_inst_id, new_id) = (|| {
					// appending to an existing inst_id is easy, creating a new inst_id means assuming game.moves to be fully
					// regenerated and checking its children.len()

					let mut clump = clump.into_iter().peekable();

					let (mut prev_inst_id, mut prev_id) =
						clump.peek().copied().unwrap_or((usize::MAX, usize::MAX));

					for (inst_id, id) in clump.chain(std::iter::once((usize::MAX, usize::MAX))) {
						if inst_id > prev_inst_id && prev_id <= max_id {
							// inst_id has an id without a foreign
							//
							// yes this is the only reason for this loop
							return (prev_inst_id, prev_id + 1);
						}

						prev_inst_id = inst_id;
						prev_id = id;
					}

					// no id holes to be filled, creating new inst_id
					let new_inst_id = game.moves.children.len();
					(new_inst_id, 0)
				})();

				*main.mut_at(x, y) = Block::Foreign(*wid, new_inst_id, new_id);

				let mut taken_moves = std::mem::take(&mut game.moves);
				match taken_moves.regenerate(game, game.main_id) {
					Ok(a) => a,
					Err(err) => {
						eprintln!("failed to regenerate world after placing a foreign\n{err}")
					}
				};
				game.moves = taken_moves;
			}
			_ => {}
		}
	}
	pub fn released(&mut self, x: i32, y: i32, game: &mut Game) {
		let main = main_or_return!(mut game);
		match self {
			Self::PlaceWire { start } => {
				if let Some(start) = start {
					let x_diff = x - start.0;
					let y_diff = y - start.1;

					let (horizontal, oldfrom, oldto) = if x_diff.abs() >= y_diff.abs() {
						(true, start.0, x)
					} else {
						(false, start.1, y)
					};

					let reverse = oldfrom > oldto;
					let from = oldfrom.min(oldto);
					let to = oldfrom.max(oldto);

					for i in from..to + 1 {
						let x = if horizontal { i } else { start.0 };
						let y = if horizontal { start.1 } else { i };

						*main.mut_at(x, y) = {
							if horizontal {
								Block::Wire(if !reverse {
									Direction::Right
								} else {
									Direction::Left
								})
							} else {
								Block::Wire(if !reverse {
									Direction::Bottom
								} else {
									Direction::Top
								})
							}
						};
					}
				};
				*start = None;
			}
			_ => {}
		}
	}
}
