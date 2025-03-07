use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{game::WorldId, processor::Instruction};

use anyhow::{anyhow, Context};

use super::stack::Stack;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ForeignRef {
	Foreign(WorldId),
}

/// Equation represents how we get a value ingame. (like outputs)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Equation {
	Input(usize),
	Or(Box<Equation>, Box<Equation>),
	Not(Box<Equation>),
	Const(bool),

	/// Foreign is special, as it can't be turned into instructions as is. \
	/// you need to convert it to a plain equation one way or another
	Foreign(ForeignRef, usize, usize, Vec<Equation>), // foreign details and input equations

	Shared(SharedStore),
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
	/// generates an equation that is only true if every equation in iter is true
	pub fn all(iter: impl Iterator<Item = Self>) -> Self {
		Self::not(Self::any(iter.map(|c_eq| Self::not(c_eq))))
	}

	pub fn map_inputs<E>(self, f: impl Fn(usize) -> Result<Self, E>) -> Result<Self, E> {
		use std::rc::Rc;
		fn internal<E, F: Fn(usize) -> Result<Equation, E>>(
			eq: Equation,
			f: Rc<F>,
		) -> Result<Equation, E> {
			match eq {
				Equation::Input(id) => f(id),
				Equation::Or(a_eq, b_eq) => Ok(Equation::or(
					internal(*a_eq, f.clone())?,
					internal(*b_eq, f)?,
				)),
				Equation::Not(n_eq) => Ok(Equation::not(internal(*n_eq, f)?)),
				Equation::Foreign(w_id, inst_id, id, in_eqs) => Ok(Equation::Foreign(
					w_id,
					inst_id,
					id,
					in_eqs
						.into_iter()
						.map(|in_eq| internal(in_eq, f.clone()))
						.collect::<Result<Vec<_>, _>>()?,
				)),
				Equation::Const(_) => Ok(eq),
				Equation::Shared(sh) => sh.store.clone().with_mut_borrow(|data| {
					let replacer_eq: Equation = unsafe { std::mem::zeroed() };
					let data_eq = std::mem::replace(&mut data.eq, replacer_eq);
					data.eq = internal(data_eq, f.clone())?;

					Ok(Equation::Shared(sh))
				}),
			}
		}
		internal(self, Rc::new(f))
	}
	pub fn map_foreigns<E>(
		self,
		f: impl Fn(ForeignRef, usize, usize, Vec<Equation>) -> Result<Self, E>,
	) -> Result<Self, E> {
		use std::rc::Rc;
		fn internal<E, F: Fn(ForeignRef, usize, usize, Vec<Equation>) -> Result<Equation, E>>(
			eq: Equation,
			f: Rc<F>,
		) -> Result<Equation, E> {
			match eq {
				Equation::Input(_) | Equation::Const(_) => Ok(eq),
				Equation::Or(a_eq, b_eq) => Ok(Equation::or(
					internal(*a_eq, f.clone())?,
					internal(*b_eq, f.clone())?,
				)),
				Equation::Not(n_eq) => Ok(Equation::not(internal(*n_eq, f)?)),
				Equation::Foreign(w_id, inst_id, id, in_eqs) => {
					let in_eqs = in_eqs
						.into_iter()
						.map(|in_eq| internal(in_eq, f.clone()))
						.collect::<Result<Vec<_>, _>>()?;
					f(w_id, inst_id, id, in_eqs)
				}
				Equation::Shared(sh) => sh.store.clone().with_mut_borrow(|data| {
					let replacer_eq: Equation = unsafe { std::mem::zeroed() };
					let data_eq = std::mem::replace(&mut data.eq, replacer_eq);
					data.eq = internal(data_eq, f.clone())?;

					Ok(Equation::Shared(sh))
				}),
			}
		}
		internal(self, Rc::new(f))
	}

	pub fn complexity(&self) -> i32 {
		fn internal(eq: &Equation, map: &mut Vec<u64>) -> i32 {
			match eq {
				&Equation::Const(_) => 0,
				&Equation::Input(_) => 1,
				Equation::Not(n_eq) => {
					if let Some((a, b)) = eq.xor_recognition() {
						internal(&a, map) + internal(&b, map) + 1
					} else if let Some(iter) = eq.and_recognition() {
						iter.fold(0, |acc, a_eq| acc + internal(a_eq, map)) + 1
					} else {
						internal(n_eq.as_ref(), map) + 1
					}
				}
				Equation::Or(a_eq, b_eq) => {
					internal(a_eq.as_ref(), map) + internal(b_eq.as_ref(), map) + 2
				}
				Equation::Shared(sh) => {
					// data.eq.complexity() + 1 on the first call to this shared, 1 for every call after that

					let mut hash = DefaultHasher::new();
					sh.store.with_borrow(|data| data.hash(&mut hash));
					let hash = hash.finish();

					if !map.contains(&hash) {
						map.push(hash);
						sh.store.with_borrow(|data| internal(&data.eq, map) + 1)
					} else {
						1
					}
				}
				Equation::Foreign(_, _, _, inputs) => {
					eprintln!("calling Equation::Foreign.complexity() will pretty much just have to guess about the inside world's complexity");
					inputs.iter().map(|a| internal(a, map)).sum::<i32>() + 5
				}
			}
		}

		// contains shareds that have already been evaluated
		let mut shared_map = Vec::<u64>::new();
		internal(self, &mut shared_map)
	}

	/// recursively `simplif`ies (optimizes) the expression
	pub fn simplify(self) -> Self {
		match self {
			Self::Input(id) => Self::Input(id),
			Self::Not(n_eq) => {
				let n_eq = *n_eq;
				let n_eq = n_eq.simplify();

				let is_and = n_eq.and_recognition().is_some();

				match n_eq {
					Self::Const(v) => Self::Const(!v),
					Self::Not(nn_eq) if !is_and => *nn_eq, //- !!v = v
					_ => Self::not(n_eq),
				}
			}
			Self::Or(a_eq, b_eq) => {
				let (a_eq, b_eq) = (*a_eq, *b_eq);
				let (a_eq, b_eq) = (a_eq.simplify(), b_eq.simplify());

				match (a_eq, b_eq) {
					(Equation::Const(true), _) | (_, Equation::Const(true)) => Self::Const(true), // if either is true self is true
					(Equation::Const(false), eq) | (eq, Equation::Const(false)) => eq, // if either is false return the other one
					(eq, Equation::Not(n_eq)) | (Equation::Not(n_eq), eq) if eq == *n_eq => {
						Self::Const(true) // x || !x = true
					}
					(a_eq, b_eq) if a_eq == b_eq => a_eq, // if a and b are the same return either one
					(a_eq, b_eq) => Self::or(a_eq, b_eq),
				}
			}
			Self::Const(v) => Self::Const(v),
			Self::Foreign(wid, inst_id, id, i_eqs) => Equation::Foreign(
				wid,
				inst_id,
				id,
				i_eqs.into_iter().map(|i_eq| i_eq.simplify()).collect(),
			),
			Equation::Shared(sh) => sh.store.clone().with_mut_borrow(|data| {
				let replacer_eq: Equation = unsafe { std::mem::zeroed() };
				let data_eq = std::mem::replace(&mut data.eq, replacer_eq);
				data.eq = data_eq.simplify();

				Equation::Shared(sh)
			}),
		}
	}

	/// shorthand for [Self::to_insts] with less technicalities
	pub fn gen_insts(
		&self,
		out_ptr: usize,
		stack_bottom: usize,
	) -> anyhow::Result<Vec<Instruction>> {
		let mut vec = vec![];
		self.to_insts(out_ptr, Stack::new(stack_bottom), &mut vec)?;
		Ok(vec)
	}

	/// if self contains shareds, make sure to call [Self::reset_shareds] if calling to_insts again for a different program
	pub fn to_insts(
		&self,
		out_ptr: usize,
		stack: Stack,
		insts: &mut Vec<Instruction>,
	) -> anyhow::Result<()> {
		match self {
			&Equation::Input(id) => insts.push(Instruction::SummonInput { id, out: out_ptr }),
			Equation::Not(n_eq) => {
				macro_rules! base_case {
					() => {{
						// base case
						n_eq.to_insts(out_ptr, stack.clone(), insts)?;
						insts.push(Instruction::Not {
							ptr: out_ptr,
							out: out_ptr,
						})
					}};
				}

				if let Some((a, b)) = self.xor_recognition() {
					let stack = stack.grow(1);
					a.to_insts(stack.top(), stack.clone(), insts)?;
					b.to_insts(out_ptr, stack.clone(), insts)?;
					insts.push(Instruction::Xor { a: stack.top(), b: out_ptr, out: out_ptr });
				} else
				// if this is an and, generate an and instruction chain for however long we need to
				if let Some(mut ands) = self.and_recognition() {
					if let Some(and_eq) = ands.next() {
						and_eq.to_insts(out_ptr, stack.grow(1), insts)?;
					} else {
						base_case!()
					}
					for and_eq in ands {
						and_eq.to_insts(stack.top(), stack.grow(1), insts)?;
						insts.push(Instruction::And {
							a: out_ptr,
							b: stack.top(),
							out: out_ptr,
						});
					}
				} else {
					base_case!()
				}
			}
			Equation::Or(_, _) => {
				let mut ors = self.collect_ors().into_iter();
				ors.next()
					.expect("this is impossible since this is an or, with a minimum of two ors")
					.to_insts(out_ptr, stack.grow(1), insts)?;
				for or_eq in ors {
					or_eq.to_insts(stack.top(), stack.grow(1), insts)?;
					insts.push(Instruction::Or {
						a: out_ptr,
						b: stack.top(),
						out: out_ptr,
					});
				}
			}
			&Equation::Const(val) => {
				insts.push(Instruction::Set { ptr: out_ptr, val });
			}
			Equation::Foreign(_, _, _, _) => {
				return Err(anyhow!("attempted to turn an Equation::Foreign into instructions. use Equation::map_foreigns"))
			}
			Equation::Shared(sh) => sh.to_insts(out_ptr, stack, insts)?,
		}
		Ok(())
	}

	pub fn reset_shareds(&self) {
		match self {
			Equation::Const(_) | Equation::Input(_) => (),
			Equation::Not(n_eq) => n_eq.reset_shareds(),
			Equation::Or(a_eq, b_eq) => {
				a_eq.reset_shareds();
				b_eq.reset_shareds();
			}
			Equation::Foreign(_, _, _, in_eqs) => {
				for in_eq in in_eqs {
					in_eq.reset_shareds();
				}
			}
			Equation::Shared(sh) => {
				sh.store.with_mut_borrow(|data| data.found_at = None);
			}
		}
	}
	pub fn reservations_internal(eq: &Equation, map: &mut Vec<u64>) -> usize {
		match eq {
			Equation::Const(_) | Equation::Input(_) => 0,
			Equation::Not(n_eq) => Self::reservations_internal(n_eq.as_ref(), map),
			Equation::Or(a_eq, b_eq) => {
				Self::reservations_internal(a_eq.as_ref(), map)
					+ Self::reservations_internal(b_eq.as_ref(), map)
			}
			Equation::Foreign(_, _, _, in_eqs) => in_eqs
				.iter()
				.map(|a| Self::reservations_internal(a, map))
				.sum(),
			Equation::Shared(sh) => {
				let mut hash = DefaultHasher::new();
				sh.store.with_borrow(|data| data.hash(&mut hash));
				let hash = hash.finish();

				if !map.contains(&hash) {
					map.push(hash);
					1 + sh
						.store
						.with_borrow(|a| Self::reservations_internal(&a.eq, map))
				} else {
					0
				}
			}
		}
	}
	#[allow(unused)]
	pub fn reservations(&self) -> usize {
		let mut map = vec![];
		Self::reservations_internal(self, &mut map)
	}

	/// if self is a an or, return every equation that if true, will turn self true \
	/// so even like nested shits and shit like that
	///
	/// if self isn't or, return `vec![&self]`
	pub fn collect_ors(&self) -> Vec<&Equation> {
		if let Equation::Or(a_eq, b_eq) = self {
			a_eq.collect_ors()
				.into_iter()
				.chain(b_eq.collect_ors())
				.collect()
		} else {
			vec![self]
		}
	}
	/// returns a list of equations that if all are true, self is true
	pub fn and_recognition(&self) -> Option<impl Iterator<Item = &Equation>> {
		if let Equation::Not(n_eq) = self {
			if let Equation::Or(_, _) = n_eq.as_ref() {
				let ors = n_eq.collect_ors();

				let is_andable = ors
					.iter()
					.map(|eq| {
						if let Equation::Not(_) = eq {
							true
						} else {
							false
						}
					})
					.filter(|p| !p) // filter only the ones that aren't a not
					.next()
					.is_none();

				if is_andable {
					return Some(ors.into_iter().map(|e| match e {
						Equation::Not(n_eq) => n_eq.as_ref(),
						_ => panic!("we just made sure everything in here is a not"),
					}));
				}
			}
		}
		None
	}
	/// if an xor, returns two equations that just need to be xor'd
	pub fn xor_recognition(&self) -> Option<(Equation, Equation)> {
		let ands = match self.and_recognition() {
			Some(iter) => {
				if iter.size_hint().0 != 2 {
					return None;
				}
				let vec = iter.collect::<Vec<_>>();
				if vec.len() != 2 {
					return None;
				} else {
					(vec[0], vec[1])
				}
			}
			None => return None,
		};
		let n_ands = (Equation::not(ands.0.clone()), Equation::not(ands.1.clone()));

		let (is_or_first, both) = {
			if let Some(ands) = n_ands.0.and_recognition() {
				(false, ands)
			} else if let Some(ands) = n_ands.1.and_recognition() {
				(true, ands)
			} else {
				return None;
			}
		};
		let both = {
			if both.size_hint().0 != 2 {
				return None;
			}
			let vec = both.collect::<Vec<_>>();
			if vec.len() != 2 {
				return None;
			}
			(vec[0].clone(), vec[1].clone())
		};

		let _either = {
			let to_check = if is_or_first { ands.0 } else { ands.1 };
			match to_check {
				Equation::Or(a, b) => {
					let is_either = |val: &Equation| -> bool { **a == *val || **b == *val };

					if is_either(&both.0) && is_either(&both.1) {
						// cool we can go ahead
					} else {
						println!("nah cause {:?}, {:?} != {:?}, {:?}", a, b, both.0, both.1);
						return None;
					}
				}
				_ => return None,
			}
		};

		Some(both)
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// this data structure is unique to every instance of [SharedData]
/// just contains a fake mutable reference to [SharedData]
pub struct SharedStore {
	pub store: sui::core::Store<SharedData>,
}
impl std::hash::Hash for SharedStore {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		// this hash implementation just calls SharedData's hash implementation.
		// this means we need to make `Shared`s artifically unique in the optimizer
		self.store.with_borrow(|a| a.hash(state))
	}
}
impl SharedStore {
	pub fn new(eq: Equation) -> Self {
		Self {
			store: sui::core::Store::new(SharedData::new(eq)),
		}
	}

	fn to_insts(
		&self,
		out_ptr: usize,
		stack: Stack,
		insts: &mut Vec<Instruction>,
	) -> anyhow::Result<()> {
		self.store
			.with_mut_borrow(|data| data.to_insts(out_ptr, stack, insts))
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// when optimizing, the optimizer recognizes equations that are calculated multiple times,
/// creates one of this data type, and links it into [Equation::Shared]
pub struct SharedData {
	found_at: Option<usize>, // Some(pointer)
	pub eq: Equation,
}
impl SharedData {
	pub fn new(eq: Equation) -> Self {
		SharedData { found_at: None, eq }
	}

	fn to_insts(
		&mut self,
		out_ptr: usize,
		stack: Stack,
		insts: &mut Vec<Instruction>,
	) -> anyhow::Result<()> {
		if let Some(at) = self.found_at {
			insts.push(Instruction::Copy {
				src_ptr: at,
				dst_ptr: out_ptr,
			});
		} else {
			let shared_out =
				stack.check_in().with_context(|| {
					format!("failed to check-in to the stack for {self:?}\ndid you forget to reserve memory for it?")
				})?;

			self.eq.to_insts(out_ptr, stack.clone(), insts)?;

			insts.push(Instruction::Copy {
				src_ptr: out_ptr,
				dst_ptr: shared_out,
			});
			self.found_at = Some(shared_out);
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_shareds() {
		let shared1 = SharedStore::new(Equation::all(
			[Equation::Input(0), Equation::Input(2)].into_iter(),
		));
		let shared1 = Equation::Shared(shared1);

		let shared2 = SharedStore::new(Equation::all(
			[Equation::Input(1), Equation::Input(3)].into_iter(),
		));
		let shared2 = Equation::Shared(shared2);

		let eq1 = Equation::all([shared1.clone(), shared2.clone()].into_iter());
		let eq2 = Equation::any([shared1.clone(), shared2.clone()].into_iter());

		let mut insts = vec![];

		let stack = Stack::with_reserved(
			2,
			Equation::any([&eq1, &eq2].into_iter().cloned()).reservations(),
		);
		eq1.to_insts(0, stack.clone(), &mut insts).expect("hey 1");
		eq2.to_insts(1, stack.clone(), &mut insts).expect("hey 2");

		use crate::processor::Memory;
		let mut mem = Memory::default();
		for zero in [false, true] {
			for one in [false, true] {
				for two in [false, true] {
					for three in [false, true] {
						mem.execute(&insts, &[zero, one, two, three]);
						dbg!(zero, one, two, three);
						assert_eq!(mem.get(0), (zero && two) && (one && three));
						println!("shared && shared worked");
						assert_eq!(mem.get(1), (zero && two) || (one && three));
						println!("shared || shared worked");
					}
				}
			}
		}
	}
}
