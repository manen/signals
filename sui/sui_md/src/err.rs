#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("io error in sui_md: {0}")]
	IO(#[from] std::io::Error),

	#[error("requested resource does not exist:\n{0}")]
	DoesNotExist(String),
}
pub type Result<T, E = Error> = std::result::Result<T, E>;
