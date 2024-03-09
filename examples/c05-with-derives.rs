use rpc_router::{RpcHandler, RpcHandlerError, RpcParams, RpcResource, RpcResourcesBuilder, RpcRouter};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::task::JoinSet;

// region:    --- Custom Error

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, RpcHandlerError, Serialize)]
pub enum Error {
	// TBC
}

impl core::fmt::Display for Error {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}

// endregion: --- Custom Error

#[derive(Clone, RpcResource)]
pub struct ModelManager;

#[derive(Deserialize, RpcParams)]
pub struct ParamsIded {
	pub id: i64,
}

pub async fn get_task(_mm: ModelManager, params: ParamsIded) -> Result<i64> {
	Ok(params.id + 9000)
}

#[tokio::main]
async fn main() -> Result<()> {
	// -- router
	let mut rpc_router: RpcRouter = RpcRouter::new();
	rpc_router = rpc_router.add_dyn("get_task", get_task.into_dyn());
	let rpc_router_base = Arc::new(rpc_router);
	let rpc_resources_base = RpcResourcesBuilder::default().insert(ModelManager).build_shared();

	// -- spawn calls
	let mut joinset = JoinSet::new();
	for _ in 0..2 {
		let rpc_router = rpc_router_base.clone();
		let rpc_resources = rpc_resources_base.clone();
		joinset.spawn(async move {
			let rpc_router = rpc_router.clone();

			let params = json!({"id": 123});

			rpc_router.call("get_task", rpc_resources, Some(params)).await
		});
	}

	// -- Print results
	while let Some(result) = joinset.join_next().await {
		println!("res: {result:?}");
	}

	Ok(())
}
