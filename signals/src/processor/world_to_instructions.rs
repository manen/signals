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

		// something breaks if the wire isn't straight
		// other than that ts works suprisingly well wth
		// couple of stack overflows but nothing some debugging can't fix

		let all_directions_except = |except: Option<Direction>| {
			let mut potential_sources = Direction::all()
				.filter(|dir| except.map(|except| *dir != except).unwrap_or(true))
				.map(|dir| (dir.reverse(), dir.rel()))
				.map(|(from, (r_x, r_y))| (from, (b_x + r_x, b_y + r_y)))
				.map(|(from, coords)| internal(&world, coords, Some(from)));

			let mut i = 0;
			let mut next_child_eq = || {
				let next = potential_sources.next();
				if let Some(next) = next {
					i += 1;
					Ok(next?)
				} else {
					None.with_context(|| {
						format!("potential_sources iterator failed to yield after {i} next calls")
					})
				}
			};

			Ok(Equation::or(
				next_child_eq()?,
				Equation::or(next_child_eq()?, next_child_eq()?),
			))
		};
		let all_directions = || {
			let from =    from.with_context(|| "blocks that could receive signals from any direction should not be called without a from argument")?;
			all_directions_except(Some(from))
		};

		match b {
			&Block::Wire(dir) => all_directions_except(Some(dir)),
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

	/// recursively `simplif`ies (optimizes) the expression
	pub fn simplify(self) -> Self {
		match self {
			Self::Input(id) => Self::Input(id),
			Self::Not(n_eq) => {
				let n_eq = *n_eq;

				match n_eq {
					Self::Const(v) => Self::Const(!v),
					Self::Not(nn_eq) => *nn_eq, //- !!v = v
					_ => Self::not(n_eq.simplify()),
				}
			}
			Self::Or(a_eq, b_eq) => {
				let (a_eq, b_eq) = (*a_eq, *b_eq);
				let (a_eq, b_eq) = (a_eq.simplify(), b_eq.simplify());

				if a_eq == Equation::Const(false) {
					return b_eq.simplify();
				}
				if b_eq == Equation::Const(false) {
					return a_eq.simplify();
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
				n_eq.to_insts(out_ptr, stack_top, insts);
				insts.push(Instruction::Not {
					ptr: out_ptr,
					out: out_ptr,
				})
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
}
