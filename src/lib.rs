// common return type
// the Error is a short description of the problem
pub type Error = &'static str;
pub type Result<T> = std::result::Result<T, Error>;

// submodules
pub mod file;
pub mod date;
pub mod event;
