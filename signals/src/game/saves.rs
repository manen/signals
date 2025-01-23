use anyhow::{anyhow, Context};
use miniz_oxide::{deflate::compress_to_vec, inflate::decompress_to_vec_with_limit};

use super::Worlds;

pub fn read_worlds(path: &str) -> anyhow::Result<Worlds> {
	load_worlds(
		&std::fs::read(path)
			.with_context(|| format!("couldn't read {path} in saves::read_worlds"))?,
	)
}
pub fn load_worlds(bytes: &[u8]) -> anyhow::Result<Worlds> {
	let decomp_bytes = match decompress_to_vec_with_limit(bytes, 60000) {
		Ok(a) => a,
		Err(err) => return Err(anyhow!("failed to decompress save bytes:\n{err}")),
	};
	Ok(bincode::deserialize(&decomp_bytes)?)
}
pub fn write_worlds(worlds: &Worlds) -> anyhow::Result<Vec<u8>> {
	let raw_bin = bincode::serialize(worlds)?;
	let comp_bin = compress_to_vec(&raw_bin, 6);
	Ok(comp_bin)
}
