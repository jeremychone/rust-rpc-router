use super::extensions::Extensions;

#[derive(Clone, Default)]
pub struct RpcResources {
	typed_map: Extensions,
}

impl RpcResources {
	pub fn get<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
		self.typed_map.get().cloned()
	}

	pub fn insert<T: Clone + Send + Sync + 'static>(mut self, val: T) -> Self {
		self.typed_map.insert(val);
		self
	}
}
