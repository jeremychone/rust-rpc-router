use crate::Result;

pub trait FromResources<K> {
	fn from_resources(rpc_resources: &K) -> Result<Self>
	where
		Self: Sized;
}
