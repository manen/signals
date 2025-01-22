use std::collections::HashMap;
use std::hash::Hash;

use crate::world::World;

pub type WorldId = uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
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
