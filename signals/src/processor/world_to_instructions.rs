use anyhow::{anyhow, Context};

use crate::{
	game::{Game, WorldId},
	processor::eq::Equation,
	world::{Block, Direction, World},
};

use super::{eq::ForeignRef, program, stack::Stack, Instruction};

/// returns none if world doesn't exist
pub fn world_to_instructions(game: &Game, world_id: WorldId) -> anyhow::Result<Vec<Instruction>> {
	let mut vec = vec![];
	let world = game
		.worlds
		.at(world_id)
		.with_context(|| format!("no world with id {world_id:?}"))?;
	let outputs_len = world.outputs().count();

	let mut program = vec![];
	for i in 0..outputs_len {
		let eq = world_output_to_eq(game, world_id, i)
			.with_context(|| format!("error while generating eq for output {i}"))?;
		program.push(eq.simplify());
	}

	let program = program::shared_recognition(program);

	let reservations = {
		let mut reservations = 0;
		let mut map = Default::default();
		for eq in program.iter() {
			let to_add = Equation::reservations_internal(eq, &mut map);
			reservations += to_add;
		}
		reservations
	};
	let stack = Stack::with_reserved(outputs_len, reservations);

	for (i, eq) in program.iter().enumerate() {
		eq.to_insts(i, stack.clone(), &mut vec)
			.with_context(|| format!("error while turning eq into insts for output {i}"))?;
	}

	Ok(vec)
}

pub fn world_output_to_eq(game: &Game, world_id: WorldId, id: usize) -> anyhow::Result<Equation> {
	let world = game
		.worlds
		.at(world_id)
		.with_context(|| "no world with id {world_id:?}")?;

	if let Some((_, coords)) = world.outputs().filter(|(this_id, _)| *this_id == id).next() {
		world_block_to_eq(game, world_id, coords)
	} else {
		Err(anyhow!("no output with id {id} in world {world_id:?}"))
	}
}

/// returns whether that given block in a world is on or off as an equation
pub fn world_block_to_eq(
	game: &Game,
	world_id: WorldId,
	coords: (i32, i32),
) -> anyhow::Result<Equation> {
	let world = game
		.worlds
		.at(world_id)
		.with_context(|| "this world does not exist")?;

	let eq = block_to_eq_internal(world, coords, None, vec![])?.simplify();

	// --- this is the part that inlines all the foreigns
	let eq = eq
		.map_foreigns(|w_id, _, id, in_eqs| {
			let w_id = match w_id {
				ForeignRef::Foreign(w_id) => w_id,
			};
			let a = match world_output_to_eq(&game, w_id, id) {
				Ok(a) => a,
				Err(err) => {
					eprintln!("{err}\nusing Const(false) instead");
					Equation::Const(false)
				}
			};
			a.map_inputs(|id| {
				let f_input = in_eqs
					.iter()
					.nth(id)
					.cloned()
					.unwrap_or(Equation::Const(false));
				anyhow::Ok(f_input)
			})
		})?
		.simplify();

	Ok(eq)
}

fn block_to_eq_internal(
	world: &World,
	(b_x, b_y): (i32, i32),
	from: Option<Direction>,
	mut circular_check: Vec<((i32, i32), Option<Direction>)>, // <- really inefficient workaround alert!!!
) -> anyhow::Result<Equation> {
	let b = if let Some(b) = world.at(b_x, b_y) {
		b
	} else {
		eprintln!("no such block in this world");
		return Ok(Equation::Const(false));
	};

	if circular_check.contains(&((b_x, b_y), from)) {
		// this case means we've already been to this block before, and now we're here again.
		// this means a circular dependency, except if this is a wire pointing a direction that doesn't matter.
		// (in which case we return const(false) anyway)
		// yeah handling edge cases is fun

		match b {
			Block::Wire(dir) if from.map(|from| from != *dir).unwrap_or(false) => {
				// doesn't even matter we'll return false in a couple of nanoseconds
			}
			_ => {
				return Err(anyhow!("this world has a circular dependency, starting from ({b_x}, {b_y})\npath taken: {circular_check:#?}"));
			}
		}
	}
	circular_check.push(((b_x, b_y), from));

	// next up is a precise specification for each block because we need feature parity between realtime and computed mode

	let all_directions_except = |except: Option<Direction>| {
		let potential_sources = Direction::all()
			.filter(|dir| except.map(|except| *dir != except).unwrap_or(true))
			.map(|dir| (dir.reverse(), dir.rel()))
			.map(|(from, (r_x, r_y))| (from, (b_x + r_x, b_y + r_y)));

		let mut eq = Equation::Const(false);
		for (from, coords) in potential_sources {
			eq = Equation::any(
				[
					eq,
					block_to_eq_internal(&world, coords, Some(from), circular_check.clone())
						.with_context(|| format!("{b_x} {b_y} -> {} {}", coords.0, coords.1))?,
				]
				.into_iter(),
			);
		}
		Ok(eq)
	};
	let all_directions = || {
		let from =    from.with_context(|| "blocks that could receive signals from any direction should not be called without a from argument")?;
		all_directions_except(Some(from))
	};

	match b {
		&Block::Wire(base_dir) => {
			if from.map(|from| from != base_dir).unwrap_or(false) {
				// if from points to a block which this wire would not actually pass a signal to
				return Ok(Equation::Const(false));
			}

			let (mut w_x, mut w_y) = (b_x, b_y);
			// we're tracing backwards so one step in base_dir.reverse() every iteration

			let mut eq = Equation::Const(false);
			loop {
				let back_dir = base_dir.reverse();
				let (r_x, r_y) = back_dir.rel();

				let behind = world.at(w_x, w_y);

				match behind {
					Some(&Block::Wire(behind_dir)) if base_dir == behind_dir => {
						let left_dir = base_dir.rotate_l();
						let right_dir = base_dir.rotate_r();
						let (left, right) = (left_dir.rel(), right_dir.rel());
						let left = (w_x + left.0, w_y + left.1);
						let right = (w_x + right.0, w_y + right.1);

						let left = block_to_eq_internal(
							world,
							left,
							Some(left_dir.reverse()),
							circular_check.clone(),
						)?;
						let right = block_to_eq_internal(
							world,
							right,
							Some(right_dir.reverse()),
							circular_check.clone(),
						)?;

						eq = Equation::any([eq, left, right].into_iter());

						w_x += r_x;
						w_y += r_y;
					}
					_ => {
						let b_eq = block_to_eq_internal(
							world,
							(w_x, w_y),
							Some(base_dir),
							circular_check.clone(),
						)?;
						break Ok(Equation::any([eq, b_eq].into_iter()));
					}
				}
			}
		}
		Block::Not(_) => {
			// nots work differently in evaluated vs real time mode

			let base = all_directions()?;
			Ok(Equation::not(base))
		}
		Block::Junction => {
			if let Some(from) = from {
				let (r_x, r_y) = from.reverse().rel();
				block_to_eq_internal(
					world,
					(b_x + r_x, b_y + r_y),
					Some(from),
					circular_check.clone(),
				)
			} else {
				Err(anyhow!(
					"tried to turn junction into eq without passing from arg"
				))
			}
		}
		Block::Router => all_directions(),
		Block::Input(id) => Ok(Equation::Input(*id)),
		Block::Switch(val) => Ok(Equation::Const(*val)),
		Block::Output(_) if from.is_none() => all_directions_except(None), // start case
		Block::Nothing | Block::Error(_) | Block::Output(_) => Ok(Equation::Const(false)),
		&Block::Foreign(wid, inst_id, id) => {
			let foreign_inputs = foreign_inputs(world, inst_id, id, from, circular_check.clone())?;

			Ok(Equation::Foreign(
				ForeignRef::Foreign(wid),
				inst_id,
				id,
				foreign_inputs,
			))
		}
	}
}

fn foreign_inputs(
	world: &World,
	inst_id: usize,
	because_id: usize,
	because_from: Option<Direction>,
	circular_check: Vec<((i32, i32), Option<Direction>)>,
) -> anyhow::Result<Vec<Equation>> {
	let foreigns = world.find_foreigns();
	let mut foreigns = foreigns
		.into_iter()
		.filter(|(_, (_, this_inst_id, _))| *this_inst_id == inst_id)
		.collect::<Vec<_>>();
	foreigns.sort_by_key(|a| a.1 .1);

	let mut vec = vec![Equation::Const(false); foreigns.len()];

	for (coords, (_, _, id)) in foreigns {
		let total_eq = {
			let directions = Direction::all().filter(|dir| {
				if id == because_id {
					because_from.map(|from| *dir != from).unwrap_or(false)
				} else {
					true
				}
			});
			let a = directions
				.map(|dir| (dir.reverse(), dir.rel()))
				.map(|(from, (r_x, r_y))| (from, (coords.0 + r_x, coords.1 + r_y)));

			let mut eq = Equation::Const(false);
			for (from, coords) in a {
				if let Block::Foreign(..) = world.at(coords.0, coords.1).unwrap_or(&Block::Nothing)
				{
					// foreigns don't pass signals to each other
				} else {
					eq = Equation::any(
						[
							eq,
							block_to_eq_internal(
								&world,
								coords,
								Some(from),
								circular_check.clone(),
							)?,
						]
						.into_iter(),
					);
				}
			}
			eq
		};

		vec[id] = total_eq;
	}

	Ok(vec)
}

#[cfg(test)]
mod tests {
	use super::super::*;
	use super::*;

	#[test]
	fn equations_and() {
		let and = Equation::not(Equation::or(
			Equation::not(Equation::Input(0)),
			Equation::not(Equation::Input(1)),
		));

		let mut insts = vec![];
		and.to_insts(0, Stack::new(2), &mut insts)
			.expect("no foreigns here");

		let mut mem = Memory::default();

		let mut run = |a: bool, b: bool| -> bool {
			mem.execute(&insts, &[a, b]);

			mem.get(0)
		};

		assert_eq!(run(false, false), false);
		assert_eq!(run(true, false), false);
		assert_eq!(run(false, true), false);
		assert_eq!(run(true, true), true);
	}

	#[test]
	fn equations_xor() {
		let xor = Equation::all(
			[
				Equation::or(Equation::Input(0), Equation::Input(1)),
				Equation::not(Equation::all(
					[Equation::Input(0), Equation::Input(1)].into_iter(),
				)),
			]
			.into_iter(),
		);

		let mut insts = vec![];
		xor.to_insts(0, Stack::new(2), &mut insts)
			.expect("no foreigns here");

		let mut mem = Memory::default();

		let mut run = |a: bool, b: bool| -> bool {
			mem.execute(&insts, &[a, b]);
			mem.get(0)
		};

		assert_eq!(run(false, false), false);
		assert_eq!(run(true, false), true);
		assert_eq!(run(false, true), true);
		assert_eq!(run(true, true), false);
	}

	#[test]
	fn test_generated_xor() {
		let insts = [
			Instruction::SummonInput { id: 0, out: 0 },
			Instruction::SummonInput { id: 1, out: 2 },
			Instruction::Or { a: 0, b: 2, out: 0 },
			Instruction::SummonInput { id: 1, out: 1 },
			Instruction::Not { ptr: 1, out: 1 },
			Instruction::SummonInput { id: 0, out: 2 },
			Instruction::Not { ptr: 2, out: 2 },
			Instruction::Or { a: 1, b: 2, out: 1 },
			Instruction::And { a: 0, b: 1, out: 0 },
		];

		let mut mem = Memory::default();

		let mut run = |a: bool, b: bool| -> bool {
			mem.execute(&insts, &[a, b]);
			mem.get(0)
		};

		assert_eq!(run(false, false), false);
		assert_eq!(run(true, false), true);
		assert_eq!(run(false, true), true);
		assert_eq!(run(true, true), false);
	}

	#[test]
	fn foreign_test() {
		let inside = Equation::all([Equation::Input(0), Equation::Input(1)].into_iter());

		let outside = Equation::Foreign(
			ForeignRef::Foreign(Default::default()),
			0,
			0,
			vec![Equation::Input(2), Equation::Input(3)],
		);

		let total: Result<_, ()> = outside.map_foreigns(|_, _, _, in_eqs| {
			inside.clone().map_inputs(|in_id| Ok(in_eqs[in_id].clone()))
		});
		let total = total.expect("oadigh");

		let insts = total
			.simplify()
			.gen_insts(0, 4)
			.expect("failed to gen instructions");

		let mut mem = Memory::default();

		let mut run = |a: bool, b: bool| -> bool {
			mem.execute(&insts, &[false, false, a, b]);
			mem.get(0)
		};

		assert_eq!(run(false, false), false);
		assert_eq!(run(true, false), false);
		assert_eq!(run(false, true), false);
		assert_eq!(run(true, true), true);
	}
}
