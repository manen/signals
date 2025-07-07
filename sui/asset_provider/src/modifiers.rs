use crate::{Assets, Error};

#[derive(Copy, Clone, Debug)]
pub struct Empty;
impl Assets for Empty {
	async fn asset(&self, key: &str) -> crate::Result<crate::Asset, crate::Error> {
		Err(Error::NoSuchAsset { tried: key.into() })
	}
}

#[derive(Clone, Debug)]
pub struct Log<A: Assets> {
	a: A,
}
impl<A: Assets> Log<A> {
	pub fn new(assets: A) -> Self {
		Self { a: assets }
	}
}
impl<A: Assets> Assets for Log<A> {
	fn asset(
		&self,
		key: &str,
	) -> impl std::future::Future<Output = crate::Result<crate::Asset, Error>> + Send + Sync {
		async fn asset<A>(
			future: impl std::future::Future<Output = crate::Result<crate::Asset, Error>>,
			key: &str,
		) -> crate::Result<crate::Asset, crate::Error> {
			let asset = future.await;

			match asset {
				Ok(a) => {
					println!("retrieved asset {key} from {}", std::any::type_name::<A>());
					Ok(a)
				}
				Err(err) => {
					eprintln!(
						"failed to retrieve asset {key} from {}\n{err}",
						std::any::type_name::<A>()
					);
					Err(err)
				}
			}
		}
		asset::<A>(self.a.asset(key), key)
	}
}
