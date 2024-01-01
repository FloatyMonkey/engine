pub mod asset;
pub mod ecs;
pub mod geometry;
pub mod gpu;
pub mod graphics;
pub mod math;
pub mod os;
pub mod scene;
pub mod time;

#[macro_use]
extern crate bitflags;

pub struct Error {
	pub error: String,
}

impl std::fmt::Debug for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.error)
	}
}

impl From<windows::core::Error> for Error {
	fn from(err: windows::core::Error) -> Error {
		Error {
			error: err.message().to_string_lossy(),
		}
	}
}
