use std::borrow::Cow;

pub const RELATIVE_TO_ROOT: &str = "./signals/assets/";

#[derive(Clone, Debug)]
/// signals::Assets will first attempt to read from disk, and use the github repo's latest commit as a fallback
pub enum Assets {
	Fs(asset_provider::FsAssets),
	Http(asset_provider::HttpAssets),
}
impl Assets {
	pub fn new() -> asset_provider::Result<Self> {
		fn process_key(key: &str) -> Cow<str> {
			Cow::Owned(format!("https://raw.githubusercontent.com/manen/signals/refs/heads/master/signals/assets/{key}"))
		}

		match asset_provider::FsAssets::new(RELATIVE_TO_ROOT) {
			Ok(fs) => Ok(Self::Fs(fs)),
			Err(err) => {
				eprintln!(
					"failed to load assets from \"{}\", using http as a fallback\n{err}",
					RELATIVE_TO_ROOT
				);
				let http = asset_provider::HttpAssets::new(process_key);
				Ok(Self::Http(http))
			}
		}
	}
}
impl asset_provider::Assets for Assets {
	async fn asset(
		&self,
		key: &str,
	) -> asset_provider::Result<asset_provider::Asset, asset_provider::Error> {
		match self {
			Assets::Fs(fs) => fs.asset(key).await,
			Assets::Http(http) => http.asset(key).await,
		}
	}
}
