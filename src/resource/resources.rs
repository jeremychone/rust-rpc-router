use super::resources_inner::ResourcesInner;
use std::sync::Arc;

// region:    --- Builder

#[derive(Debug, Default, Clone)]
pub struct ResourcesBuilder {
	pub(crate) resources_inner: ResourcesInner,
}

impl ResourcesBuilder {
	pub fn get<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
		self.resources_inner.get().cloned()
	}

	pub fn append<T: Clone + Send + Sync + 'static>(mut self, val: T) -> Self {
		self.resources_inner.insert(val);
		self
	}

	/// Convenient append method to avoid moving out value.
	/// Use `.append(val)` if not sure.
	pub fn append_mut<T: Clone + Send + Sync + 'static>(&mut self, val: T) {
		self.resources_inner.insert(val);
	}

	/// Build `Resources` with an `Arc` to enable efficient cloning without
	/// duplicating the content (i.e., without cloning the type hashmap).
	pub fn build(self) -> Resources {
		Resources::from_base_inner(self.resources_inner)
	}
}

// endregion: --- Builder

// region:    --- Resources

#[derive(Debug, Clone, Default)]
pub struct Resources {
	base_inner: Arc<ResourcesInner>,
	overlay_inner: Arc<ResourcesInner>,
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
		// first additional, then base
		self.overlay_inner.get::<T>().or_else(|| self.base_inner.get::<T>()).cloned()
	}

	pub fn is_empty(&self) -> bool {
		self.base_inner.is_empty() && self.overlay_inner.is_empty()
	}
}

// -- Privates
impl Resources {
	/// Build a resource from a base_inner ResourcesInner
	/// This is called bac the ResourcesBuilder
	pub(crate) fn from_base_inner(base_inner: ResourcesInner) -> Self {
		Self {
			base_inner: Arc::new(base_inner),
			overlay_inner: Default::default(),
		}
	}

	pub(crate) fn new_with_overlay(&self, overlay_resources: Resources) -> Self {
		Self {
			base_inner: self.base_inner.clone(),
			overlay_inner: overlay_resources.base_inner.clone(),
		}
	}
}

// endregion: --- Resources
