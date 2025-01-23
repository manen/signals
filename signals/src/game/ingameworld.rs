use anyhow::{anyhow, Context};
use std::collections::HashMap;

use crate::{
	game::{Game, WorldId},
	processor,
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

/// IngameWorld is a recursive structure that contains the moves (instance) of the world it's pointing to in id \
/// this is needed to make every world have unique instances of worlds, to be contained in foreigns
#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub struct IngameWorld {
	pub world_id: WorldId,
	pub moves: Vec<Move>,
	pub children: Vec<IngameWorld>,
}
impl IngameWorld {
	/// generates the ingameworld required for a World to have foreigns working \
	/// see [IngameWorld::regenerate]
	pub fn generate(game: &mut Game, world_id: WorldId) -> anyhow::Result<Self> {
		let mut ingameworld = Self::default();
		ingameworld.regenerate(game, world_id)?;
		Ok(ingameworld)
	}
	/// regenerates itself and fixes the world if needed \
	/// recursive
	pub fn regenerate(&mut self, game: &mut Game, world_id: WorldId) -> anyhow::Result<()> {
		// oh god i hate leaving comments like this but i have NO CLUE what is happening this
		// is some true dogshit code really should've left some comments

		let world = match game.worlds.at(world_id) {
			Some(a) => a,
			None => {
				return Err(anyhow!(
					"could not find a world with id {world_id}\nneeded for IngameWorld::regenerate"
				))
			}
		};
		let mut foreigns = world.find_foreigns();
		foreigns.sort_by(|(_, (_, a_inst_id, a_id)), (_, (_, b_inst_id, b_id))| {
			(a_inst_id * 1000 + a_id).cmp(&(b_inst_id * 1000 + b_id))
		});

		let mut inst_ids_already_done =
			Vec::with_capacity((self.children.len() as i32 - 1 as i32).max(0) as usize);
		let mut next_id_per_inst_id: HashMap<usize, usize> = HashMap::new();

		for (coords, (inst_world_id, mut inst_id, id)) in foreigns {
			if !inst_ids_already_done.contains(&inst_id) {
				// only happens once per inst_id
				let inst = match self.children.iter_mut().nth(inst_id) {
					Some(ptr) => ptr,
					None => {
						if inst_id != self.children.len() {
							inst_id = self.children.len();
							game.worlds
								.at_mut(world_id)
								.with_context(|| "impossible case in IngameWorld::regenerate")?
								.map_at(coords.0, coords.1, |_| {
									Block::Foreign(inst_world_id, inst_id, id)
								})
						}
						self.children.push(IngameWorld {
							world_id: inst_world_id,
							..Default::default()
						});
						&mut self.children[inst_id]
					}
				};

				if inst_world_id == world_id {
					println!("deleting a foreign referencing the world the foreign's in (world_id: {world_id:?})");
					game.worlds
						.at_mut(world_id)
						.with_context(|| "second impossible case in IngameWorld::regenerate")?
						.map_at(coords.0, coords.1, |_| Block::Error(BlockError::Recursion));
					continue;
				}

				inst.regenerate(game, inst_world_id)?;
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
						.mut_at(coords.0, coords.1) = Block::Error(BlockError::WorldDoesntExist);
					return Ok(()); // <- fake Ok
				}
			};

			let next = match next_id_per_inst_id.get(&inst_id) {
				Some(next) => *next,
				None => 0,
			};
			let max_id = inst_world.inputs_count().max(inst_world.outputs_count());
			let world_mut = game
				.worlds
				.at_mut(world_id)
				.with_context(|| "impossible case 3 IngameWorld::regenerate()")?;
			if id > max_id {
				eprintln!("the world (id: {world_id:?}) contained a foreign that exceeded the maximum possible id of {max_id} for the world given ({inst_world_id:?}) by being {id}");
				world_mut.map_at(coords.0, coords.1, |_| {
					Block::Error(BlockError::MaxIdExceeded)
				});
			} else {
				world_mut.map_at(coords.0, coords.1, |_| {
					Block::Foreign(inst_world_id, inst_id, next)
				});
				next_id_per_inst_id.insert(inst_id, next + 1);
			}
		}
		Ok(())
	}

	fn tick(&mut self, game: &mut Game, ret: impl FnMut(Move)) -> anyhow::Result<()> {
		let new_moves = game
			.worlds
			.at_mut(self.world_id)
			.with_context(|| format!("this IngameWorld points to a nonexistent world\n{self:#?}"))?
			.tick(std::mem::take(&mut self.moves), |_, _, _| {});
		self.process_moves(new_moves, ret, &game.programs, &mut game.memory);
		Ok(())
	}
	pub(crate) fn tick_children(&mut self, game: &mut Game) -> anyhow::Result<()> {
		for (i, child) in self.children.iter_mut().enumerate() {
			child.tick(game, |m| match m {
				Move::Output { id, .. } => self.moves.push(Move::Foreign {
					inst_id: i,
					id,
					signal: Signal::ExternalPoweron,
				}),
				mov => eprintln!("only outputs should be returned from child worlds ({mov:?})"),
			})?;
			child.tick_children(game)?;
		}
		Ok(())
	}

	fn child_mut(&mut self, inst_id: usize) -> &mut Self {
		if self.children.len() as i32 - 1 <= inst_id as i32 {
		} else {
			self.children
				.resize(self.children.len().max(inst_id + 1), Default::default());
		}
		&mut self.children[inst_id]
	}

	pub(super) fn process_moves(
		&mut self,
		new_moves: Vec<Move>,
		mut ret: impl FnMut(Move),
		programs: &super::Programs,
		memory: &mut processor::Memory,
	) {
		self.moves = Vec::with_capacity(new_moves.len());

		let push_unique = |moves: &mut Vec<_>, mov: Move| {
			if !moves.contains(&mov) {
				moves.push(mov);
			}
		};
		let mut collected_foreign_inputs = Vec::<(usize, usize)>::new();

		// the code that pushes foreigns into the child ingw:
		//				push_unique(
		//					&mut self.child_mut(inst_id).moves,
		//					Move::Input {
		//						id,
		//						signal: Signal::ExternalPoweron,
		//					},
		//				)

		for mov in new_moves {
			match mov {
				Move::Inside { .. } => push_unique(&mut self.moves, mov),
				Move::Output { .. } => ret(mov),
				Move::Foreign { inst_id, id, .. } => {
					collected_foreign_inputs.push((inst_id, id));
				}
				Move::Input { .. } => {
					eprintln!("unexpected input move in moves processing: {mov:?}")
				}
			}
		}

		let (processorable_inst_ids) = {
			let mut insts = collected_foreign_inputs
				.iter()
				.map(|(inst_id, _)| (*inst_id))
				.collect::<Vec<_>>();
			insts.dedup();

			let processorable_inst_ids = insts
				.into_iter()
				.filter_map(|inst_id| {
					self.children
						.iter()
						.nth(inst_id)
						.map(|a| (inst_id, a.world_id))
				})
				.filter(|(_, wid)| match programs.get(wid) {
					Some((Some(_), _, _)) => true,
					_ => false,
				})
				.collect::<HashMap<_, _>>();

			(processorable_inst_ids)
		};

		let processor_inputs = {
			// this part turns and organizes the collected foreign inputs into a HashMap<inst_id, (inputs_vec, outputs_count)>
			let mut collected_foreign_inputs = collected_foreign_inputs
				.into_iter()
				.filter(|(inst_id, _)| processorable_inst_ids.get(&inst_id).is_some())
				.collect::<Vec<_>>();
			collected_foreign_inputs.sort_by(|(a_inst_id, a_id), (b_inst_id, b_id)| {
				(*a_inst_id * 1000 + a_id).cmp(&(b_inst_id * 1000 + b_id))
			});

			let mut map = HashMap::<usize, Vec<bool>>::new();

			for (inst_id, id) in collected_foreign_inputs.iter() {
				if let Some(inputs) = map.get_mut(&inst_id) {
					match inputs.iter_mut().nth(*id) {
						Some(a) => *a = true,
						None => eprintln!("ignoring true input for foreign {inst_id}:{id} because the generated inputs vec is too short\ncheck the part right under this line in the else block"),
					}
				} else {
					let wid = processorable_inst_ids.get(inst_id).expect("impossible case, nothing in collected_foreign_inputs here that hasn't been programified");
					let inputs_len = match programs.get(wid) {
						Some((_, inputs_len, _)) => *inputs_len,
						_ => panic!("this case is pretty impossible"), // im just gonna leave it like this gl
					};
					let mut vec = vec![false; inputs_len];
					if let Some(b) = vec.iter_mut().nth(*id) {
						*b = true;
					} else {
						eprintln!("dropped a foreign signal because for some fucking reason this shit doesn't work")
					}

					map.insert(*inst_id, vec);
				}
			}

			map
		};

		for (inst_id, inputs) in processor_inputs.iter() {
			let wid = match processorable_inst_ids.get(&inst_id) {
				Some(a) => a,
				None => continue, // <- impossible case i think
			};
			let (insts, _, outputs_len) = match programs.get(wid) {
				Some((Some(a), inputs_len, outputs_len)) => (a, *inputs_len, *outputs_len),
				None | Some((None, _, _)) => {
					eprintln!(
					"IMPOSSIBLE CASE we already checked if there's a program for this wid ({wid})"
				);
					continue;
				}
			};

			memory.execute(&insts, &inputs);

			for i in 0..outputs_len {
				if memory.get(i) {
					push_unique(
						&mut self.moves,
						Move::Foreign {
							inst_id: *inst_id,
							id: i,
							signal: Signal::ExternalPoweron,
						},
					)
				}
			}
		}
	}
}
