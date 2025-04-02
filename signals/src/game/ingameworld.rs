use anyhow::{anyhow, Context};
use std::{
	collections::HashMap,
	hash::{DefaultHasher, Hasher},
};

use crate::{
	game::{Game, WorldId},
	world::{Block, BlockError, Move, Signal},
};
use std::hash::Hash;

// how are we gonna get a whole world inside a block? i hear you asking
// create a block that contains: world id, input id, output id
// for now we can just hardcode for example signals from the left will be treated as input,
// while all other sides will output the output
// this is actually feasible but i'm soooooo tired gn

// we need a new architecture this one sucks ass
// ingame world:
// - World
// - moves
// - children (Vec<ingame world>)

// this makes everything easier let's implement it
// so long Game struct (it stayed)

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum IngameWorldType {
	Simulated {
		moves: Vec<Move>,
	},
	Processor {
		inputs: Vec<bool>,
		prev_in_hash: u64,
		prev_out: Vec<bool>,
	},
}
impl Default for IngameWorldType {
	fn default() -> Self {
		Self::simulated()
	}
}
impl IngameWorldType {
	pub const fn simulated() -> Self {
		Self::Simulated { moves: vec![] }
	}
	pub const fn processor() -> Self {
		Self::Processor {
			inputs: vec![],
			prev_in_hash: 0,
			prev_out: vec![],
		}
	}
}

/// IngameWorld represents a world inside of a block, either fully simulated
/// or calculated on demand
#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub struct IngameWorld {
	pub world_id: WorldId,
	pub typ: IngameWorldType,
	pub children: Vec<IngameWorld>,
}
impl IngameWorld {
	/// generates the ingameworld required for a World to have foreigns working \
	/// see [IngameWorld::regenerate] \
	/// will return simulated if world_id == game.main_id \
	/// if you need an ingameworld that is surely simulated, use [IngameWorld::simulated]
	pub fn generate(game: &mut Game, world_id: WorldId) -> anyhow::Result<Self> {
		if game.main_id == world_id {
			return Self::simulated(game, world_id);
		}

		let typ = {
			if let Some((Some(_), _, _)) = game.programs.get(&world_id) {
				IngameWorldType::processor()
			} else {
				IngameWorldType::simulated()
			}
		};
		let mut ingameworld = Self {
			world_id,
			typ,
			..Default::default()
		};
		ingameworld.regenerate(game, world_id)?;
		Ok(ingameworld)
	}
	pub fn simulated(game: &mut Game, world_id: WorldId) -> anyhow::Result<Self> {
		let mut ingameworld = Self {
			world_id,
			typ: IngameWorldType::simulated(),
			..Default::default()
		};
		ingameworld.regenerate(game, world_id)?;
		Ok(ingameworld)
	}

	/// regenerates itself and fixes the world if needed \
	/// recursive \
	/// will not edit self.typ, make sure you have that set up correctly
	pub fn regenerate(&mut self, game: &mut Game, world_id: WorldId) -> anyhow::Result<()> {
		match &mut self.typ {
			IngameWorldType::Simulated { .. } => {
				let world = match game.worlds.at(world_id) {
					Some(a) => a,
					None => {
						return Err(anyhow!(
							"could not find a world with id {world_id}\nneeded for IngameWorld::regenerate"
						))
					}
				};
				let mut foreigns = world.find_foreigns().collect::<Vec<_>>();
				foreigns.sort_by(|(_, (_, a_inst_id, a_id)), (_, (_, b_inst_id, b_id))| {
					(a_inst_id * 1000 + a_id).cmp(&(b_inst_id * 1000 + b_id))
				});

				// yeah so we have to make sure there's no holes in the inst_ids

				let mut replaced_inst_ids = HashMap::<usize, usize>::new();

				let mut prev_inst_id: i32 = -1;
				for (coords, (f_wid, inst_id, id)) in foreigns.iter().copied() {
					if let Some(new_inst_id) = replaced_inst_ids.get(&inst_id) {
						if let Some(a) = game.worlds.at_mut(f_wid) {
							*a.mut_at(coords.0, coords.1) =
								Block::Foreign(f_wid, *new_inst_id as usize, id);
						}
						continue;
					}
					if prev_inst_id == inst_id as i32 {
						continue;
					}
					if inst_id as i32 == prev_inst_id + 1 {
						prev_inst_id = inst_id as i32;
						continue;
					}
					if inst_id as i32 > prev_inst_id + 1 {
						prev_inst_id += 1;
						replaced_inst_ids.insert(inst_id, prev_inst_id as usize);
						if let Some(a) = game.worlds.at_mut(f_wid) {
							*a.mut_at(coords.0, coords.1) =
								Block::Foreign(f_wid, prev_inst_id as usize, id);
						}
					}
				}

				let foreigns = foreigns.into_iter().map(|(coords, (f_wid, inst_id, id))| {
					(
						coords,
						(
							f_wid,
							replaced_inst_ids.get(&inst_id).copied().unwrap_or(inst_id),
							id,
						),
					)
				});

				let mut inst_ids_already_done = Vec::with_capacity(self.children.len());
				let mut next_id_per_inst_id: HashMap<usize, usize> = HashMap::new();

				for (coords, (inst_world_id, inst_id, id)) in foreigns {
					if !inst_ids_already_done.contains(&inst_id) {
						// only happens once per inst_id

						let inst = match self.children.iter_mut().nth(inst_id) {
							Some(ptr) => ptr,
							None => {
								// generate ingameworld for inst_id
								// since foreigns is already sorted by inst_id and id, it's a little easier
								assert_eq!(self.children.len(), inst_id); // <- make sure just pushing it onto self.children will work

								let child = IngameWorld::generate(game, inst_world_id)?;
								self.children.push(child);
								&mut self.children[inst_id]
							}
						};

						if inst_world_id == world_id {
							println!("deleting a foreign referencing the world the foreign's in (world_id: {world_id:?})");
							game.worlds
								.at_mut(world_id)
								.with_context(|| {
									"second impossible case in IngameWorld::regenerate"
								})?
								.map_at(coords.0, coords.1, |_| {
									Block::Error(BlockError::Recursion { inst_id, id })
								});
							continue;
						}

						inst.regenerate(game, inst_world_id)
							.with_context(|| "error while regenerating child")?;
						inst_ids_already_done.push(inst_id);
					}

					let inst_world = match game.worlds.at(inst_world_id) {
						Some(a) => a,
						None => {
							eprintln!("deleting a foreign referencing an invalid world ({inst_world_id}) in {world_id}");
							// this foreign has an invalid world id
							*game
								.worlds
								.at_mut(world_id)
								.with_context(|| "impossible case 3 in IngameWorld::regenerate()")?
								.mut_at(coords.0, coords.1) = Block::Error(BlockError::WorldDoesntExist {
								wid: inst_world_id,
								inst_id,
								id,
							});
							return Ok(()); // <- fake Ok
						}
					};

					let next = match next_id_per_inst_id.get(&inst_id) {
						Some(next) => *next,
						None => 0,
					};
					let max_id = inst_world.max_f_id();
					let world_mut = game
						.worlds
						.at_mut(world_id)
						.with_context(|| "impossible case 3 IngameWorld::regenerate()")?;
					if id > max_id {
						eprintln!("the world (id: {world_id:?}) contained a foreign that exceeded the maximum possible id of {max_id} for the world given ({inst_world_id:?}) by being {id}");
						world_mut.map_at(coords.0, coords.1, |block| {
							Block::Error(match block {
								Block::Foreign(wid, inst_id, id) => BlockError::MaxIdExceeded {
									wid,
									this_was: id,
									max_id,
								},
								_ => BlockError::Other,
							})
						});
					} else {
						world_mut.map_at(coords.0, coords.1, |_| {
							Block::Foreign(inst_world_id, inst_id, next)
						});
						next_id_per_inst_id.insert(inst_id, next + 1);
					}
				}
			}
			IngameWorldType::Processor { inputs, .. } => {
				if self.children.len() != 0 {
					self.children = vec![];
				}
				if let Some((Some(_), in_len, _)) = game.programs.get(&world_id) {
					if inputs.len() != *in_len {
						*inputs = vec![false; *in_len];
					}
				} else {
					// if there's no program for this world, turn self into a simulated
					self.typ = IngameWorldType::simulated();
					self.regenerate(game, world_id)?;
				}
			}
		}

		Ok(())
	}

	pub fn tick(
		&mut self,
		game: &mut Game,
		mut ret: impl FnMut(Move),
		set_dt: bool,
	) -> anyhow::Result<()> {
		match &mut self.typ {
			IngameWorldType::Simulated { moves } => {
				let new_moves = game.worlds.at_mut(self.world_id).with_context(|| {
					format!("this IngameWorld points to a nonexistent world\nworld_id: {:?}\ntyp: {:?}\nchildren: {:#?}", self.world_id, "IngameWorldType::Simulated", self.children)
				})?;
				let new_moves = new_moves.tick(std::mem::take(moves), |x, y, dt| {
					if set_dt {
						*game.drawmap.mut_at(x, y) = game
							.drawmap
							.at(x, y)
							.copied()
							.unwrap_or_default()
							.apply_new(dt);
					}
				});
				self.process_moves(new_moves, ret);
			}
			IngameWorldType::Processor {
				inputs,
				prev_in_hash,
				prev_out,
			} => {
				let (insts, _, out_len) = match game.programs.get(&self.world_id) {
					Some((Some(insts), in_len, out_len)) => (insts, in_len, out_len),
					_ => {
						self.regenerate(game, self.world_id).with_context(|| {
							format!("while regenerating ingameworld for {}", self.world_id)
						})?;
						return Ok(());
						// regenerate will turn self into a simulated world
					}
				};

				let mut in_hash = DefaultHasher::new();
				inputs.hash(&mut in_hash);
				let in_hash = in_hash.finish();

				if in_hash != *prev_in_hash {
					game.memory.execute(&insts, &inputs);
					for i in 0..*out_len {
						if game.memory.get(i) {
							ret(Move::Output {
								id: i,
								signal: Signal::Default,
							});
						}
					}

					*prev_in_hash = in_hash;
					*prev_out = game.memory[0..*out_len].iter().copied().collect();
				} else {
					for (i, val) in prev_out.iter().enumerate() {
						if *val {
							ret(Move::Output {
								id: i,
								signal: Signal::Default,
							})
						}
					}
				}

				*inputs = inputs.into_iter().map(|_| false).collect();
			}
		}
		Ok(())
	}
	pub(crate) fn tick_children(&mut self, game: &mut Game) -> anyhow::Result<()> {
		match &mut self.typ {
			IngameWorldType::Simulated { moves } => {
				for (i, child) in self.children.iter_mut().enumerate() {
					child
						.tick(
							game,
							|m| match m {
								Move::Output { id, .. } => moves.push(Move::Foreign {
									inst_id: i,
									id,
									signal: Signal::ExternalPoweron,
								}),
								mov => {
									eprintln!(
									"only outputs should be returned from child worlds ({mov:?})"
								)
								}
							},
							false,
						)
						.with_context(|| {
							format!("from #{i} child of world_id: {:?}", self.world_id)
						})?;
					child.tick_children(game)?;
				}
			}
			IngameWorldType::Processor { .. } => {}
		}
		Ok(())
	}

	pub(super) fn process_moves(&mut self, new_moves: Vec<Move>, mut ret: impl FnMut(Move)) {
		match &mut self.typ {
			IngameWorldType::Simulated { moves } => {
				*moves = Vec::with_capacity(new_moves.len());

				for mov in new_moves {
					match mov {
						Move::Inside { .. } => self.receive_moves([mov]),
						Move::Output { .. } => ret(mov),
						Move::Foreign { inst_id, id, .. } => {
							self.children.iter_mut().nth(inst_id).map(|f| {
								f.receive_moves([Move::Input {
									id,
									signal: Signal::ExternalPoweron,
								}])
							});
						}
						Move::Input { .. } => {
							eprintln!("unexpected input move in moves processing: {mov:?}")
						}
					}
				}
			}
			IngameWorldType::Processor { .. } => {}
		}
	}
	/// receive moves from parent or self, with dedup and everything handled inside
	pub fn receive_moves(&mut self, new_moves: impl IntoIterator<Item = Move>) {
		match &mut self.typ {
			IngameWorldType::Simulated { moves } => {
				let new_moves = new_moves.into_iter();
				moves.reserve(new_moves.size_hint().0.max(0));
				for mov in new_moves {
					if !moves.contains(&mov) {
						moves.push(mov);
					}
				}
			}
			IngameWorldType::Processor { inputs, .. } => {
				for mov in new_moves {
					match mov {
						Move::Input { id, .. } => {
							if inputs.len() - 1 < id {
								inputs.resize(id + 1, false);
							}
							inputs[id] = true;
						}
						_ => eprintln!(
							"dropping non-input move to a a processor ingameworld:\n{mov:#?}"
						),
					}
				}
				0;
			}
		}
	}
}
