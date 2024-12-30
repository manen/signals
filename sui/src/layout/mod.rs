use std::fmt::Debug;

use crate::Details;
use raylib::prelude::RaylibDrawHandle;

pub mod page;
pub use page::Page;

pub trait Layable {
	fn size(&self) -> (i32, i32);
	fn render(&self, d: &mut RaylibDrawHandle, det: Details, scale: f32);

	/// this function is called by the parent of this component \
	/// return events to be bubbled back
	fn pass_event(&self, event: Event) -> Option<Event>;
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Event {
	MouseEvent {
		x: i32,
		y: i32,
	},

	/// use these to bubble
	Named {
		/// id is meant to be a general identifier of what this event's about
		id: &'static str,
		/// n could be anything you want, probably most useful as an array index
		n: i32,
	},
}

/// DynamicLayable is like dyn Layable but better
pub struct DynamicLayable {
	/// heap pointer, allocated with std::alloc
	ptr: *mut u8,
	layout: std::alloc::Layout,
	type_name: &'static str,

	size: fn(*const u8) -> (i32, i32),
	render: fn(*const u8, d: &mut RaylibDrawHandle, det: Details, scale: f32),
	pass_event: fn(*const u8, event: Event) -> Option<Event>,

	drop: fn(*mut u8),
	clone: Option<fn(*const u8, std::alloc::Layout) -> *mut u8>,
	debug: Option<fn(*const u8) -> String>,
}
// memory stuff for DynamicLayable
impl DynamicLayable {
	pub fn new<L: Layable + Debug + Clone>(layable: L) -> Self {
		Self::new_notraits(layable)
			.add_debug::<L>()
			.add_clone::<L>()
	}
	pub fn new_only_debug<L: Layable + Debug>(layable: L) -> Self {
		Self::new_notraits(layable).add_debug::<L>()
	}
	pub fn new_only_clone<L: Layable + Clone>(layable: L) -> Self {
		Self::new_notraits(layable).add_clone::<L>()
	}

	// common trait impls: might cause some really ugly stuff to happen if L != the L new_notraits was called with
	fn add_debug<L: Layable + Debug>(mut self) -> Self {
		// no pretty printed version cause what do u even need that for
		fn debug<L: Layable + Debug>(ptr: *const u8) -> String {
			let b: &L = unsafe { &*(ptr as *const L) };
			format!("{b:?}")
		}

		self.debug = Some(debug::<L>);
		self
	}
	fn add_clone<L: Layable + Clone>(mut self) -> Self {
		/// clone allocates a new pointer for layout and copies layout.size() bytes from ptr into it, returning the new ptr \
		/// things might get really ugly if layout != self.layout, manual memory management is scary
		fn clone<L: Layable + Clone>(ptr: *const u8, layout: std::alloc::Layout) -> *mut u8 {
			let b: &L = unsafe { &*(ptr as *const L) };
			let clone = L::clone(b);

			let new_ptr = unsafe { std::alloc::alloc(layout) };
			unsafe {
				std::ptr::copy_nonoverlapping(
					&clone as *const L as *const u8,
					new_ptr,
					layout.size(),
				)
			};
			std::mem::forget(clone);

			new_ptr
		}

		self.clone = Some(clone::<L>);
		self
	}

	pub fn new_notraits<L: Layable>(layable: L) -> Self {
		let type_name = std::any::type_name::<L>();
		let layout = std::alloc::Layout::new::<L>();
		let ptr = unsafe { std::alloc::alloc(layout) } as *mut L;
		// copy contents of layable into ptr
		unsafe { std::ptr::copy_nonoverlapping(&layable as *const L, ptr, 1) };
		std::mem::forget(layable);
		let ptr = ptr as *mut u8;

		fn size<L: Layable>(ptr: *const u8) -> (i32, i32) {
			L::size(unsafe { &*(ptr as *const L) })
		}
		fn render<L: Layable>(ptr: *const u8, d: &mut RaylibDrawHandle, det: Details, scale: f32) {
			L::render(unsafe { &*(ptr as *const L) }, d, det, scale)
		}
		fn pass_event<L: Layable>(ptr: *const u8, event: Event) -> Option<Event> {
			L::pass_event(unsafe { &*(ptr as *const L) }, event)
		}

		fn drop<L: Layable>(ptr: *mut u8) {
			let mut layable: std::mem::MaybeUninit<L> = std::mem::MaybeUninit::uninit();
			unsafe { std::ptr::copy_nonoverlapping(ptr as *const L, layable.as_mut_ptr(), 1) };
			unsafe { layable.assume_init_drop() };
		}

		let d = Self {
			ptr,
			layout,
			type_name,
			size: size::<L>,
			render: render::<L>,
			pass_event: pass_event::<L>,
			drop: drop::<L>,
			clone: None,
			debug: None,
		};
		d.null_check();
		d
	}

	/// null_check panics if self.ptr is null and that's it
	fn null_check(&self) {
		if self.ptr as *const _ == std::ptr::null() {
			panic!(
				"DynamicLayable for type {} ended up having null as self.ptr, halting execution",
				self.type_name
			)
		}
	}

	/// basically [Self::new] but backwards
	pub fn take<L: Layable>(self) -> Option<L> {
		if self.can_take::<L>() {
			let mut layable: std::mem::MaybeUninit<L> = std::mem::MaybeUninit::uninit();
			unsafe { std::ptr::copy_nonoverlapping(self.ptr as *const L, layable.as_mut_ptr(), 1) };
			let layable = unsafe { layable.assume_init() };
			Some(layable)
		} else {
			None
		}
	}
	/// returns whether calling self.take<L> will return Some \
	/// only here because Self::take takes by value not by reference
	pub fn can_take<L: Layable>(&self) -> bool {
		// this is our bulletproof type-checking
		std::any::type_name::<L>() == self.type_name
			&& self.layout == std::alloc::Layout::new::<L>()
	}
}
impl Drop for DynamicLayable {
	fn drop(&mut self) {
		(self.drop)(self.ptr);
		unsafe { std::alloc::dealloc(self.ptr as *mut u8, self.layout) };
	}
}
// layable impl
impl Layable for DynamicLayable {
	fn size(&self) -> (i32, i32) {
		(self.size)(self.ptr)
	}
	fn render(&self, d: &mut RaylibDrawHandle, det: Details, scale: f32) {
		(self.render)(self.ptr, d, det, scale)
	}
	fn pass_event(&self, event: Event) -> Option<Event> {
		(self.pass_event)(self.ptr, event)
	}
}
// common trait impls
impl Debug for DynamicLayable {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self.debug {
			None => write!(f, "[DynamicLayable {}]", self.type_name),
			Some(dbgf) => {
				let s = dbgf(self.ptr);
				let type_name = self.type_name;
				fn none_or_some<T>(x: Option<T>) -> &'static str {
					match x {
						Some(_) => "Some",
						None => "None",
					}
				}
				let (clone, debug) = (none_or_some(self.clone), none_or_some(self.debug));
				write!(
					f,
					"[DynamicLayable {type_name} {s}, clone: {clone}, debug: {debug}]"
				)
			}
		}
	}
}
impl Clone for DynamicLayable {
	fn clone(&self) -> Self {
		match self.clone {
			None => panic!("attempted to clone a DynamicLayable that didn't implement cloning\nmake sure to use DynamicLayable::new or DynamicLayable::new_only_clone\nsorry for panicking but the only other option is memory corruption so i think u still got a good deal"),
			Some(clonef) => {
				let new_ptr = clonef(self.ptr, self.layout);

				Self {
					ptr: new_ptr,
					layout: self.layout,
					type_name: self.type_name,
					size: self.size,
					render: self.render,
					pass_event: self.pass_event,
					drop: self.drop,
					clone: self.clone,
					debug: self.debug
				}
			}
		}
	}
}

#[cfg(test)]
mod dynamiclayable_tests {
	use super::*;

	#[test]
	fn test_assert() {
		eprintln!("begin assert testing DynamicLayable");
		test_single(crate::text(
			"hello i'm just testing to see if all this raw memory shit broke or nah",
			100,
		));
		test_single(crate::page(vec![
			crate::text("hellop", 1),
			crate::text("hi".to_owned(), 14),
			crate::text("yessirski", 54),
		]));
		test_single(Page::new(
			vec![
				crate::text("hellop", 1),
				crate::text("hi".to_owned(), 14),
				crate::text("yessirski", 54),
			],
			false,
		));
	}
	fn test_single<L: Layable + Clone + Debug>(l: L) {
		let d = DynamicLayable::new(l.clone());
		println!("{d:?}");
		test_pair(l, d);
	}
	fn test_pair<A: Layable, B: Layable>(a: A, b: B) {
		let test_event = Event::Named {
			id: "testing",
			n: 16,
		};

		assert_eq!(a.size(), b.size());
		assert_eq!(a.pass_event(test_event), b.pass_event(test_event));
	}

	#[test]
	fn test_clone() {
		let d_a = DynamicLayable::new(crate::text("hi", 16));
		let d_b = d_a.clone();

		test_pair(d_a, d_b);

		let xample = String::from("starting value");

		let d_c = DynamicLayable::new(crate::page(vec![
			crate::text(&xample, 1),
			crate::text("hi", 14),
			crate::text("yessirski", 54),
		]));
		let d_d = d_c.clone();

		test_pair(d_c, d_d);
	}

	static mut DROPPED: bool = false;

	#[test]
	fn test_drop() {
		#[derive(Clone, Debug)]
		struct Dummy;
		impl Layable for Dummy {
			fn size(&self) -> (i32, i32) {
				(200, 200)
			}
			fn render(&self, _: &mut RaylibDrawHandle, _: Details, _: f32) {}
			fn pass_event(&self, event: Event) -> Option<Event> {
				Some(event)
			}
		}
		impl Drop for Dummy {
			fn drop(&mut self) {
				unsafe { DROPPED = true };
			}
		}

		{
			let d = DynamicLayable::new(Dummy);
		}
		assert!(unsafe { DROPPED });
	}

	#[test]
	fn test_take() {
		#[derive(Clone, Debug, PartialEq, Eq)]
		struct Dummy(i32);
		impl Layable for Dummy {
			fn size(&self) -> (i32, i32) {
				(200, 200)
			}
			fn render(&self, _: &mut RaylibDrawHandle, _: Details, _: f32) {}
			fn pass_event(&self, event: Event) -> Option<Event> {
				Some(event)
			}
		}

		let d = DynamicLayable::new(Dummy(30));
		let d_cloned = d.clone();

		assert!(!d_cloned.can_take::<crate::Comp>());
		assert!(!d_cloned.can_take::<crate::Text>());

		assert_eq!(d_cloned.take(), Some(Dummy(30)));
	}
}
