use anyhow::{anyhow, Context};

use crate::{
	game::Game,
	world::{Block, Direction, PushMoveTo, Signal, World},
};

use super::Instruction;

/// returns none if world doesn't exist
pub fn world_to_instructions(
	game: &Game,
	world_id: Option<usize>,
) -> anyhow::Result<Vec<Instruction>> {
	let mut vec = vec![];
	let world = game.world_opt(world_id).with_context(|| "no such world")?;

	// let eqs = world.outputs().map(|coords, id| output_to_eq(world, ));

	// start from the outputs:
	// go back, and we need to write this shit out backwards
	// for example, for a simple and gate:

	// output = flipped(flipped(input_0) || flipped(input_1))

	if let Some((_, coords)) = world.outputs().filter(|(id, _)| *id == 0).next() {
		let eq = world_block_to_eq(game, world_id, coords).expect("fail");
		eq.to_insts(0, 0, &mut vec);
	}

	Ok(vec)
}

/// returns whether that given block in a world is on or off as an equation
pub fn world_block_to_eq(
	game: &Game,
	world_id: Option<usize>,
	coords: (i32, i32),
) -> anyhow::Result<Equation> {
	fn internal(
		world: &World,
		(b_x, b_y): (i32, i32),
		from: Option<Direction>,
	) -> anyhow::Result<Equation> {
		let b = if let Some(b) = world.at(b_x, b_y) {
			b
		} else {
			eprintln!("no such block in this world");
			return Ok(Equation::Const(false));
		};

		// next up is a precise specification for each block because we need feature parity between realtime and computed mode

		let all_directions_except = |except: Option<Direction>| {
			let potential_sources = Direction::all()
				.filter(|dir| except.map(|except| *dir != except).unwrap_or(true))
				.map(|dir| (dir.reverse(), dir.rel()))
				.map(|(from, (r_x, r_y))| (from, (b_x + r_x, b_y + r_y)));

			let mut eq = Equation::Const(false);
			for (from, coords) in potential_sources {
				eq = Equation::any([eq, internal(&world, coords, Some(from))?].into_iter());
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

							let left = internal(world, left, Some(left_dir.reverse()))?;
							let right = internal(world, right, Some(right_dir.reverse()))?;

							eq = Equation::any([eq, left, right].into_iter());

							w_x += r_x;
							w_y += r_y;
						}
						_ => {
							let b_eq = internal(world, (w_x, w_y), Some(base_dir))?;
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
					internal(world, (b_x + r_x, b_y + r_y), Some(from))
				} else {
					Err(anyhow!(
						"tried to turn junction into eq without passing from arg"
					))
				}
			}
			Block::Router => all_directions(),
			Block::Input(id) => Ok(Equation::Input(*id)),
			Block::Switch(val) => Ok(Equation::Const(*val)),
			Block::Nothing | Block::Error(_) => Ok(Equation::Const(false)),
			Block::Output(_) => all_directions_except(None),
			Block::Foreign(_, _, _) => Err(anyhow!(
				"foreigns are not yet implented for programification"
			)),
		}
	}

	let world = game
		.world_opt(world_id)
		.with_context(|| "this world does not exist")?;
	Ok(internal(world, coords, None)?.simplify())
}

/// Equation represents how we get a value ingame. (like outputs)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Equation {
	Input(usize),
	Or(Box<Equation>, Box<Equation>),
	Not(Box<Equation>),
	Const(bool),
}
impl Equation {
	pub fn or(a: Equation, b: Equation) -> Self {
		Self::Or(Box::new(a), Box::new(b))
	}
	pub fn not(val: Equation) -> Self {
		Equation::Not(Box::new(val))
	}

	pub fn any(iter: impl Iterator<Item = Self>) -> Self {
		let mut eq = Equation::Const(false);

		for new_eq in iter {
			eq = Self::or(eq, new_eq)
		}
		eq.simplify()
	}

	/// recursively `simplif`ies (optimizes) the expression
	pub fn simplify(self) -> Self {
		match self {
			Self::Input(id) => Self::Input(id),
			Self::Not(n_eq) => {
				let n_eq = *n_eq;
				let n_eq = n_eq.simplify();

				match n_eq {
					Self::Const(v) => Self::Const(!v),
					Self::Not(nn_eq) => *nn_eq, //- !!v = v
					_ => Self::not(n_eq),
				}
			}
			Self::Or(a_eq, b_eq) => {
				let (a_eq, b_eq) = (*a_eq, *b_eq);
				let (a_eq, b_eq) = (a_eq.simplify(), b_eq.simplify());

				if [&a_eq, &b_eq]
					.into_iter()
					.filter(|a| **a == Equation::Const(true))
					.next()
					.is_some()
				{
					// if any of them are const(true)
					return Equation::Const(true);
				}

				if a_eq == Equation::Const(false) {
					return b_eq;
				}
				if b_eq == Equation::Const(false) {
					return a_eq;
				}
				Self::or(a_eq, b_eq)
			}
			Self::Const(v) => Self::Const(v),
		}
	}

	/// stack_top is where the empty memory starts
	pub fn to_insts(&self, out_ptr: usize, stack_top: usize, insts: &mut Vec<Instruction>) {
		match self {
			&Equation::Input(id) => insts.push(Instruction::SummonInput { id, out: out_ptr }),
			Equation::Not(n_eq) => {
				// if this is an and block, generate an and instruction

				// extract this and recognition part into its own thing, cause it looks really ugly rn in like the
				// most important function in the whole file
				if let Equation::Or(a_eq, b_eq) = n_eq.as_ref() {
					if let Equation::Not(an_eq) = a_eq.as_ref() {
						if let Equation::Not(bn_eq) = b_eq.as_ref() {
							an_eq.to_insts(stack_top, stack_top + 2, insts);
							bn_eq.to_insts(stack_top + 1, stack_top + 2, insts);
							insts.push(Instruction::And {
								a: stack_top,
								b: stack_top + 1,
								out: out_ptr,
							});
							return;
						}
					}
				}
				{
					// base case
					n_eq.to_insts(out_ptr, stack_top, insts);
					insts.push(Instruction::Not {
						ptr: out_ptr,
						out: out_ptr,
					})
				}
			}
			Equation::Or(a_eq, b_eq) => {
				a_eq.to_insts(stack_top, stack_top + 2, insts);
				b_eq.to_insts(stack_top + 1, stack_top + 2, insts);

				insts.push(Instruction::Or {
					a: stack_top,
					b: stack_top + 1,
					out: out_ptr,
				})
			}
			&Equation::Const(val) => {
				insts.push(Instruction::Set { ptr: out_ptr, val });
			}
		}
	}
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
		and.to_insts(0, 2, &mut insts);

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
		let make_and = |a: Equation, b: Equation| -> Equation {
			Equation::not(Equation::or(Equation::not(a), Equation::not(b)))
		};

		let xor = make_and(
			Equation::or(Equation::Input(0), Equation::Input(1)),
			Equation::not(make_and(Equation::Input(0), Equation::Input(1))),
		);

		let mut insts = vec![];
		xor.to_insts(0, 2, &mut insts);

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

	// #[test]
	// fn test_generated_xor() {
	// 	let insts = [
	// 		Instruction::SummonInput { id: 1, out: 2 },
	// 		Instruction::Not { ptr: 2, out: 2 },
	// 		Instruction::SummonInput { id: 0, out: 3 },
	// 		Instruction::Not { ptr: 3, out: 3 },
	// 		Instruction::Or { a: 2, b: 3, out: 0 },
	// 		Instruction::SummonInput { id: 0, out: 2 },
	// 		Instruction::SummonInput { id: 1, out: 3 },
	// 		Instruction::Or { a: 2, b: 3, out: 1 },
	// 		Instruction::And { a: 0, b: 1, out: 0 },
	// 	];

	// 	let mut mem = Memory::default();

	// 	let mut run = |a: bool, b: bool| -> bool {
	// 		mem.execute(&insts, &[a, b]);
	// 		mem.get(0)
	// 	};

	// 	assert_eq!(run(false, false), false);
	// 	assert_eq!(run(true, false), true);
	// 	assert_eq!(run(false, true), true);
	// 	assert_eq!(run(true, true), false);
	// }
}
