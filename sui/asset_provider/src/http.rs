use std::borrow::Cow;

use crate::{Asset, Assets, Result};

#[derive(Clone, Debug)]
/// HttpAssets allows you to get assets from the web on-demand
pub struct HttpAssets {
	process_key: fn(&str) -> Cow<str>,
}
impl HttpAssets {
	pub fn new(process_key: fn(&str) -> Cow<str>) -> Self {
		Self { process_key }
	}
}
impl Assets for HttpAssets {
	async fn asset(&self, key: &str) -> Result<Asset> {
		let url = (self.process_key)(key);
		let resp = reqwest::get(url.as_ref()).await?;
		let resp = resp.bytes().await?;
		let resp = resp.to_vec();

		Ok(Asset::new(resp))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_http() {
		test_http_impl()
	}
	#[tokio::main]
	async fn test_http_impl() {
		fn process_key(key: &str) -> Cow<str> {
			Cow::Borrowed(key)
		}
		let assets = HttpAssets::new(process_key);

		let asset = assets.asset("https://github.com/manen/signals/raw/c52733bb1d3608c700c6c364de2b1fd7645e163c/Cargo.toml").await.expect("failed to get asset");
		let asset = asset.as_str().expect("downloaded asset is not utf-8");

		assert_eq!(asset, "[package]\nname = \"signals\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nraylib = \"5.0.2\"\n")
	}
}
