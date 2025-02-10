use std::collections::HashMap;
use std::hash::Hash;

use serde::{Deserialize, Serialize};

use crate::world::World;

pub type WorldId = uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Worlds {
	worlds: HashMap<WorldId, World>,
}
impl Worlds {
	pub fn push(&mut self, world: World) -> WorldId {
		let uuid = uuid::Uuid::new_v4();
		self.worlds.insert(uuid, world);

		uuid
	}
	pub fn remove(&mut self, id: WorldId) -> Option<World> {
		self.worlds.remove(&id)
	}

	pub fn at(&self, id: WorldId) -> Option<&World> {
		self.worlds.get(&id)
	}
	pub fn at_mut(&mut self, id: WorldId) -> Option<&mut World> {
		self.worlds.get_mut(&id)
	}

	pub fn iter(&self) -> impl Iterator<Item = (&WorldId, &World)> {
		self.worlds.iter()
	}
}
impl Hash for Worlds {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		for (id, w) in self.worlds.iter() {
			id.hash(state);
			w.hash(state);
		}
	}
}

use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
/// WorldsTree is the structure that stores the categories \
/// the worlds themselves are in [Worlds]
pub struct WorldsTree {
	pub name: Cow<'static, str>,
	pub color: Option<sui::Color>,

	pub categories: Vec<WorldsTree>,
	pub worlds: Vec<WorldId>,
}
impl WorldsTree {
	/// returns all the worlds there are in this tree recursively
	pub fn worlds_r(&self) -> impl Iterator<Item = WorldId> {
		self.worlds
			.iter()
			.copied()
			.chain(self.categories.iter().map(|cat| cat.worlds_r()).flatten())
			.collect::<Vec<_>>()
			.into_iter()
	}

	pub fn categories_with_others(&self, worlds: &Worlds) -> impl Iterator<Item = Cow<WorldsTree>> {
		let mut others = worlds.worlds.keys().copied().collect::<Vec<_>>();
		for exists in self.worlds_r() {
			if let Some(i) = others.iter().position(|&x| x == exists) {
				others.remove(i);
			}
		}

		self.categories
			.iter()
			.map(|cat| Cow::Borrowed(cat))
			.chain(std::iter::once(Cow::Owned(WorldsTree {
				name: "others".into(),
				color: None,
				categories: vec![],
				worlds: others,
			})))

		// this should work next up is the ui itself

		// i'm thinking the categories on the left, where categories inside a category are rendered to the right,
		// the rest of the space could be just like the current worlds bar.
		// to store the current category/subcategory we have open, we might just use a Vec<index>

		// tried doing this something doesnt work
	}
}
