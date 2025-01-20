use crate::processor::Instruction;

use anyhow::anyhow;

/// Equation represents how we get a value ingame. (like outputs)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Equation {
	Input(usize),
	Or(Box<Equation>, Box<Equation>),
	Not(Box<Equation>),
	Const(bool),

	/// Foreign is special, as it can't be turned into instructions as is. \
	/// you need to convert it to a plain equation one way or another
	Foreign(Option<usize>, usize, usize, Vec<Equation>), // foreign details and input equations
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
			}
		}
		internal(self, Rc::new(f))
	}
	pub fn map_foreigns<E>(
		self,
		f: impl Fn(Option<usize>, usize, usize, Vec<Equation>) -> Result<Self, E>,
	) -> Result<Self, E> {
		use std::rc::Rc;
		fn internal<E, F: Fn(Option<usize>, usize, usize, Vec<Equation>) -> Result<Equation, E>>(
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
			}
		}
		internal(self, Rc::new(f))
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
		}
	}

	pub fn gen_insts(&self, out_ptr: usize, stack_top: usize) -> anyhow::Result<Vec<Instruction>> {
		let mut vec = vec![];
		self.to_insts(out_ptr, stack_top, &mut vec)?;
		Ok(vec)
	}
	/// stack_top is where the empty memory starts
	pub fn to_insts(
		&self,
		out_ptr: usize,
		stack_top: usize,
		insts: &mut Vec<Instruction>,
	) -> anyhow::Result<()> {
		match self {
			&Equation::Input(id) => insts.push(Instruction::SummonInput { id, out: out_ptr }),
			Equation::Not(n_eq) => {
				macro_rules! base_case {
					() => {{
						// base case
						n_eq.to_insts(out_ptr, stack_top, insts)?;
						insts.push(Instruction::Not {
							ptr: out_ptr,
							out: out_ptr,
						})
					}};
				}

				// if this is an and, generate an and instruction chain for however long we need to
				if let Some(mut ands) = self.and_recognition() {
					if let Some(and_eq) = ands.next() {
						and_eq.to_insts(out_ptr, stack_top + 1, insts)?;
					} else {
						base_case!()
					}
					for and_eq in ands {
						and_eq.to_insts(stack_top, stack_top + 1, insts)?;
						insts.push(Instruction::And {
							a: out_ptr,
							b: stack_top,
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
					.to_insts(out_ptr, stack_top + 1, insts)?;
				for or_eq in ors {
					or_eq.to_insts(stack_top, stack_top + 1, insts)?;
					insts.push(Instruction::Or {
						a: out_ptr,
						b: stack_top,
						out: out_ptr,
					});
				}
			}
			&Equation::Const(val) => {
				insts.push(Instruction::Set { ptr: out_ptr, val });
			}
			Equation::Foreign(_, _, _, _) => {
				return Err(anyhow!("attempted to turn an Equation::Foreign into instructions. this is impossible, as context is needed about the world inside."))
			}
		}
		Ok(())
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
}
