use super::extensions::Extensions;
use std::sync::Arc;

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

	/// Build a new `RpcResources`.
	/// Use the `build_shared` function.
	pub fn build(self) -> RpcResources {
		self.build_shared()
	}

	/// Build `RpcResources` with an `Arc` to enable efficient cloning without
	/// duplicating the content (i.e., without cloning the type hashmap).
	///
	/// Note: Use this option if each RPC call uses the same RPC Resources.
	///       If uncertain, opt for this method via `.build()`.
	pub fn build_shared(self) -> RpcResources {
		RpcResources::new_shared(self.type_map)
	}

	/// Build an owned `RpcResources`, meaning that cloning it will duplicate the entire type hashmap.
	///
	/// Note: Use this option if each RPC call requires different Rpc Resources.
	pub fn build_owned(self) -> RpcResources {
		RpcResources::new_owned(self.type_map)
	}
}

// endregion: --- Builder

#[derive(Clone)]
pub struct RpcResources {
	type_map_holder: TypeMapHolder,
}

impl RpcResources {
	pub fn get<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
		let type_map = self.type_map_holder.get();
		type_map.get().cloned()
	}
}

impl RpcResources {
	fn new_shared(type_map: Extensions) -> Self {
		Self {
			type_map_holder: TypeMapHolder::Shared(Arc::new(type_map)),
		}
	}
	fn new_owned(type_map: Extensions) -> Self {
		Self {
			type_map_holder: TypeMapHolder::Owned(type_map),
		}
	}
}

// region:    --- TypeMapHolder

#[derive(Clone)]
enum TypeMapHolder {
	Shared(Arc<Extensions>),
	Owned(Extensions),
}

impl TypeMapHolder {
	fn get(&self) -> &Extensions {
		match self {
			Self::Shared(extensions) => extensions.as_ref(),
			Self::Owned(extensions) => extensions,
		}
	}
}

// endregion: --- TypeMapHolder
