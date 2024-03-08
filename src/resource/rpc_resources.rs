use super::extensions::Extensions;

// region:    --- Builder

#[derive(Default)]
pub struct RpcResourcesBuilder {
	type_map: Extensions,
}

impl RpcResourcesBuilder {
	pub fn get<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
		self.type_map.get().cloned()
	}

	pub fn insert<T: Clone + Send + Sync + 'static>(mut self, val: T) -> Self {
		self.type_map.insert(val);
		self
	}

	pub fn build(self) -> RpcResources {
		RpcResources {
			type_map: self.type_map,
		}
	}
}

// endregion: --- Builder

#[derive(Clone)]
pub struct RpcResources {
	type_map: Extensions,
}

impl RpcResources {
	pub fn get<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
		self.type_map.get().cloned()
	}
}
