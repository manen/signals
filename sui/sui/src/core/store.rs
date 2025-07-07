use std::{
	cell::RefCell,
	hash::{DefaultHasher, Hash, Hasher},
	rc::Rc,
};

/// single threaded store
#[derive(Debug, PartialEq, Eq, Default)]
pub struct Store<T> {
	rc: Rc<RefCell<T>>,
}
impl<T> Clone for Store<T> {
	fn clone(&self) -> Self {
		Self {
			rc: self.rc.clone(),
		}
	}
}
impl<T> Store<T> {
	pub fn new(val: T) -> Self {
		Store {
			rc: Rc::new(RefCell::new(val)),
		}
	}

	pub fn with_borrow<R>(&self, f: impl FnOnce(&T) -> R) -> R {
		let rf = self.rc.borrow();
		f(&rf)
	}
	pub fn with_mut_borrow<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
		let mut rf = self.rc.borrow_mut();
		f(&mut rf)
	}

	/// sets value to val, returning the overwritten value
	pub fn set(&self, mut val: T) -> T {
		self.with_mut_borrow(|existing| std::mem::swap(existing, &mut val));
		val
	}
}
impl<T: Copy> Store<T> {
	/// copies the value and returns it
	pub fn get(&self) -> T {
		self.with_borrow(|a| *a)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_store() {
		let store = Store::new("HI!!!!");

		store.with_borrow(|a| assert_eq!(*a, "HI!!!!"));
		store.with_mut_borrow(|a| *a = "sum else");
		store.with_borrow(|a| assert_eq!(*a, "sum else"));
	}

	#[test]
	fn test_store_with_i32() {
		let store = Store::new(16);

		let num = store.get();
		assert_eq!(num, 16);

		store.with_mut_borrow(|a| *a = 13);
		let num = store.get();
		assert_eq!(num, 13);
	}
}

// ---

#[derive(Copy, Clone)]
pub struct Cached<T> {
	hash: u64,
	val: Option<T>,
}
impl<T> Default for Cached<T> {
	fn default() -> Self {
		Self { hash: 0, val: None }
	}
}
impl<T> Cached<T> {
	/// re-runs f and returns a borrow to its return value if args isn't the args this function was run with before
	pub fn update<A: PartialEq + Hash>(&mut self, args: A, f: impl Fn(A) -> T) -> &T {
		let mut hasher = DefaultHasher::new();
		args.hash(&mut hasher);
		let new_hash = hasher.finish();

		if self.hash != new_hash || self.val.is_none() {
			self.val = Some(f(args));
		}
		if let Some(val) = &self.val {
			val
		} else {
			panic!("self.val is none, even though we just checked if it's none and set it to some if it is")
		}
	}
	/// same as [Self::update], but has an unchecked args field which goes... well unchecked.
	pub fn update_with_unchecked<CA: PartialEq + Hash, UCA>(
		&mut self,
		checked_args: CA,
		unchecked_args: UCA,
		f: impl Fn(CA, UCA) -> T,
	) -> &T {
		let mut hasher = DefaultHasher::new();
		checked_args.hash(&mut hasher);
		let new_hash = hasher.finish();

		if self.hash != new_hash || self.val.is_none() {
			self.val = Some(f(checked_args, unchecked_args));
			self.hash = new_hash;
		}
		if let Some(val) = &self.val {
			val
		} else {
			panic!("self.val is none, even though we just checked if it's none and set it to some if it is")
		}
	}

	pub fn borrow(&self) -> Option<&T> {
		self.val.as_ref()
	}
}

// --

// use crate::{Comp, Compatible};

// /// Arg represents an argument to a functional component, caching its component output and skipping
// /// regenerating if the argument stayed the same
// pub struct Arg<T> {
// 	cache: Cached<Comp<'static>>,
// 	_a: std::marker::PhantomData<T>,
// }
// impl<T> Arg<T> {
// 	pub fn with_borrow<C: Compatible<'static>>(&self, f: impl FnOnce(T) -> C) -> &Comp<'static> {
// 		self.cache.update(args, f)
// 	}
// }
