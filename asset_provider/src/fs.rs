use crate::{Asset, Assets, Error, Result};
use std::{fs, io::ErrorKind, path::PathBuf};

/// implementor of [crate::Assets] that reads assets straight from the filesystem
pub struct FsAssets {
	dir: PathBuf,
}
impl FsAssets {
	pub fn new(dir: impl Into<PathBuf>) -> Result<Self> {
		let dir: PathBuf = dir.into();

		if dir.exists() {
			Ok(Self { dir })
		} else {
			Err(Error::NoAssetsDir {
				tried: dir.to_string_lossy().into(),
			})
		}
	}
}
impl Assets for FsAssets {
	async fn asset(&self, key: &str) -> Result<crate::Asset, Error> {
		// async isn't really implemented here
		match fs::read(self.dir.join(key)) {
			Ok(a) => Ok(Asset::new(a)),
			Err(err) => match err.kind() {
				ErrorKind::NotFound => Err(Error::NoSuchAsset { tried: key.into() }),
				_ => Err(Error::IO(err)),
			},
		}
	}
}
