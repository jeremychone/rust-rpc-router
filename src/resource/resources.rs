use super::store::Store;
use std::sync::Arc;

// region:    --- Builder

#[derive(Default, Clone)]
pub struct ResourcesBuilder {
	type_map: Store,
}

impl ResourcesBuilder {
	pub fn get<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
		self.type_map.get().cloned()
	}

	pub fn append<T: Clone + Send + Sync + 'static>(mut self, val: T) -> Self {
		self.type_map.insert(val);
		self
	}

	/// Build `Resources` with an `Arc` to enable efficient cloning without
	/// duplicating the content (i.e., without cloning the type hashmap).
	pub fn build(self) -> Resources {
		Resources::new_shared(self.type_map)
	}
}

// endregion: --- Builder

// region:    --- Resources

#[derive(Clone)]
pub struct Resources {
	shared_store: Arc<Store>,
}

// -- Builder
impl Resources {
	/// Returns a new `ResourcesBuilder`.
	/// This is equivalent to calling `Resources::default()`.
	pub fn builder() -> ResourcesBuilder {
		ResourcesBuilder::default()
	}
}

// -- Public Methods
impl Resources {
	pub fn get<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
		self.shared_store.get().cloned()
	}
}

// -- Privates
/// Note: For now, we only support the shared mode (i.e., Arc<Store>).
///       However, eventually, we might support an owned version.
impl Resources {
	fn new_shared(type_map: Store) -> Self {
		Self {
			shared_store: Arc::new(type_map),
		}
	}
}

// endregion: --- Resources
