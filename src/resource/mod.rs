// region:    --- Modules

mod from_resources;
mod resources;
mod resources_builder_macro;
mod resources_inner;

// -- Flatten
pub use from_resources::*;
pub use resources::*;
pub(crate) use resources_inner::ResourcesInner;

// endregion: --- Modules
