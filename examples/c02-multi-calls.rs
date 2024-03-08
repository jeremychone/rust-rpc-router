pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.

use rpc_router::{
	FromResourcesError, FromRpcResources, IntoRpcParams, RpcHandler, RpcResources, RpcResourcesBuilder, RpcRouter,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tokio::task::JoinSet;

#[derive(Clone)]
pub struct ModelManager;

impl FromRpcResources for ModelManager {
	fn from_resources(rpc_resources: &RpcResources) -> rpc_router::FromResourcesResult<Self> {
		rpc_resources
			.get::<ModelManager>()
			.ok_or_else(FromResourcesError::resource_not_found::<Self>)
	}
}

#[derive(Deserialize)]
pub struct ParamsIded {
	pub id: i64,
}
impl IntoRpcParams for ParamsIded {}

pub async fn get_task(_mm: ModelManager, params: ParamsIded) -> rpc_router::Result<i64> {
	Ok(params.id + 9000)
}

#[tokio::main]
async fn main() -> Result<()> {
	println!("Hello, world!");

	// -- router
	let mut rpc_router: RpcRouter = RpcRouter::new();
	rpc_router = rpc_router.add_dyn("get_task", get_task.into_dyn());
	let rpc_router_base = Arc::new(rpc_router);

	let mut joinset = JoinSet::new();

	let rpc_router = rpc_router_base.clone();
	joinset.spawn(async move {
		let rpc_router = rpc_router.clone();
		let rr = RpcResourcesBuilder::default().insert(ModelManager).build_owned();
		let params = json!({"id": 123});

		rpc_router.call("get_task", rr, Some(params)).await
	});

	let rpc_router = rpc_router_base.clone();
	joinset.spawn(async move {
		let rpc_router = rpc_router.clone();
		let rr = RpcResourcesBuilder::default().insert(ModelManager).build_owned();
		let params = json!({"id": 123});

		rpc_router.call("get_task", rr, Some(params)).await
	});

	// Wait for all tasks to finish
	while let Some(result) = joinset.join_next().await {
		println!("->> {result:?}");
	}

	Ok(())
}
