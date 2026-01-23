// Typestate Example: Demonstrates various invariant patterns
// that should be detected by the Design Patterns Agent

mod file_handle;
mod database;
mod resource;
mod builder;

pub use file_handle::{FileHandle, Closed, Open};
pub use database::{Connection, Disconnected, Connected};
pub use resource::Resource;
pub use builder::{Builder, BuilderEmpty, BuilderWithName, BuilderComplete};
