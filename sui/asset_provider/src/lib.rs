use core::str;
use std::{borrow::Cow, future::Future, str::Utf8Error};

#[cfg(feature = "fs")]
pub mod fs;
#[cfg(feature = "fs")]
pub use fs::FsAssets;

#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "http")]
pub use http::HttpAssets;

pub mod modifiers;
pub use modifiers::{Empty, Log};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("attempted to access an asset that doesn't exist\n{tried}")]
	NoSuchAsset { tried: String },
	#[error("failed to read assets: directory {tried} does not exist")]
	NoAssetsDir { tried: String },
	#[cfg(feature = "fs")]
	#[error("io error: {0}")]
	IO(#[from] std::io::Error),
	#[cfg(feature = "http")]
	#[error("http error: {0}")]
	Reqwest(#[from] reqwest::Error),
}
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// the powerhouse of asset_provider
pub trait Assets {
	fn asset(&self, key: &str) -> impl Future<Output = Result<Asset, Error>> + Send + Sync;
}

#[derive(Clone, Debug)]
pub struct Asset {
	bin: Cow<'static, [u8]>,
}
impl Asset {
	pub fn new(bin: impl Into<Cow<'static, [u8]>>) -> Self {
		let bin = bin.into();
		Self { bin }
	}

	pub fn as_str(self) -> Result<Cow<'static, str>, Utf8Error> {
		match self.bin {
			Cow::Borrowed(bytes) => {
				let s = std::str::from_utf8(bytes)?;
				Ok(Cow::Borrowed(s))
			}
			Cow::Owned(bytes) => {
				let s = String::from_utf8(bytes)
					.map_err(|e| std::str::Utf8Error::from(e.utf8_error()))?;
				Ok(Cow::Owned(s))
			}
		}
	}
}
impl AsRef<[u8]> for Asset {
	fn as_ref(&self) -> &[u8] {
		self.bin.as_ref()
	}
}
