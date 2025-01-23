use std::{
	collections::HashMap,
	hash::{DefaultHasher, Hash, Hasher},
};

use crate::processor::eq::SharedStore;

use super::eq::Equation;

pub type Program = Vec<Equation>;

/// i didn't make this function play smart in the case there's already shareds inside, it'll
/// probably create nested shareds which make no sense
pub fn shared_recognition(mut program: Program) -> Program {
	let mut appearances = vec![(0_u64, 0_i32); 0]; // (hash, complexity)

	for eq in program.iter() {
		add_to_appearances(eq, &mut appearances);
	}

	let appearances = {
		let mut appearances_map = HashMap::<u64, (i32, i32)>::new(); // K: hash, V: (appearances, complexity)
		for (hash, complexity) in appearances.iter().copied() {
			appearances_map.insert(
				hash,
				(
					appearances_map.get(&hash).copied().unwrap_or((0, 0)).0 + 1,
					complexity,
				),
			);
		}
		appearances_map
	};

	fn should_share(appearances: i32, complexity: i32) -> bool {
		appearances > 1 && complexity > 5
		// 5 is like not(or(input, input))
	}

	let to_share = appearances
		.iter()
		.filter(|(_, (appearances, complexity))| should_share(*appearances, *complexity))
		.map(|(hash, _)| *hash)
		.collect::<Vec<_>>();
	let mut shareds = HashMap::<u64, SharedStore>::new();

	for eq in program.iter_mut() {
		replace_with_shared_if_needed(eq, &to_share, &mut shareds);
	}
	program
}

fn replace_with_shared_if_needed(
	eq: &mut Equation,
	to_share: &[u64],
	shareds: &mut HashMap<u64, SharedStore>,
) {
	let hash = qh(eq);
	match eq {
		Equation::Not(n_eq) => replace_with_shared_if_needed(n_eq.as_mut(), to_share, shareds),
		Equation::Or(a_eq, b_eq) => {
			replace_with_shared_if_needed(a_eq.as_mut(), to_share, shareds);
			replace_with_shared_if_needed(b_eq.as_mut(), to_share, shareds);
		}
		Equation::Foreign(_, _, _, in_eqs) => {
			for in_eq in in_eqs {
				replace_with_shared_if_needed(in_eq, to_share, shareds);
			}
		}
		Equation::Shared(_) => eprintln!("unexpected shared while finding sharables: {eq:#?}"),
		Equation::Const(_) | Equation::Input(_) => (),
	}

	if to_share.contains(&hash) {
		let zeroed = unsafe { std::mem::zeroed() };
		let eq_here = std::mem::replace(eq, zeroed);

		let shstore = shareds.get(&hash).cloned().unwrap_or_else(|| {
			let new_store = SharedStore::new(eq_here);
			shareds.insert(hash, new_store.clone());
			new_store
		});
		*eq = Equation::Shared(shstore);
	}
}

fn add_to_appearances(eq: &Equation, appearances: &mut Vec<(u64, i32)>) {
	let hash = qh(eq);
	let included_before = (|| {
		for (appearance_hash, _) in appearances.iter() {
			if *appearance_hash == hash {
				return true;
			}
		}
		false
	})();
	appearances.push((hash, eq.complexity()));

	if !included_before {
		match eq {
			Equation::Not(n_eq) => add_to_appearances(n_eq, appearances),
			Equation::Or(a_eq, b_eq) => {
				add_to_appearances(a_eq.as_ref(), appearances);
				add_to_appearances(b_eq.as_ref(), appearances);
			}
			Equation::Foreign(_, _, _, in_eqs) => {
				for in_eq in in_eqs {
					add_to_appearances(in_eq, appearances);
				}
			}
			Equation::Shared(sh) => sh
				.store
				.with_borrow(|data| add_to_appearances(&data.eq, appearances)),
			Equation::Const(_) | Equation::Input(_) => (),
		}
	}
}

fn qh<T: Hash>(x: &T) -> u64 {
	let mut hash = DefaultHasher::new();
	x.hash(&mut hash);
	hash.finish()
}
