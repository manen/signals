use super::Worlds;

pub fn read_worlds(path: &str) -> anyhow::Result<Worlds> {
	load_worlds(&std::fs::read(path)?)
}
pub fn load_worlds(bytes: &[u8]) -> anyhow::Result<Worlds> {
	Ok(bincode::deserialize(bytes)?)
}
pub fn write_worlds(worlds: &Worlds) -> anyhow::Result<Vec<u8>> {
	Ok(bincode::serialize(worlds)?)
}
