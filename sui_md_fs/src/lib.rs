use std::path::PathBuf;

use sui_md::{Error, NavProvider};

#[derive(Clone, Debug)]
pub struct FsProvider {
	base: PathBuf,
}
impl FsProvider {
	pub fn new(base: PathBuf) -> Self {
		Self { base }
	}
}
impl NavProvider for FsProvider {
	fn page(&self, path: &str) -> sui_md::Result<String> {
		let path_full = self.base.join(path).canonicalize()?;
		if path_full.exists() {
			let txt = std::fs::read_to_string(&path_full)?;
			Ok(txt)
		} else {
			Err(Error::DoesNotExist(path_full.to_string_lossy().into()))
		}
	}
}
