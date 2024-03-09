pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.

use rpc_router::{FromRpcResources, IntoRpcParams, RpcHandler, RpcResourcesBuilder, RpcRouter};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tokio::task::JoinSet;

#[derive(Clone)]
pub struct ModelManager;
impl FromRpcResources for ModelManager {}

#[derive(Deserialize)]
pub struct ParamsIded {
	pub id: i64,
}
impl IntoRpcParams for ParamsIded {}

pub async fn get_task(_mm: ModelManager, params: ParamsIded) -> rpc_router::RpcHandlerResult<i64> {
	Ok(params.id + 9000)
}

#[tokio::main]
async fn main() -> Result<()> {
	// -- router
	let mut rpc_router: RpcRouter = RpcRouter::new();
	rpc_router = rpc_router.add_dyn("get_task", get_task.into_dyn());
	let rpc_router_base = Arc::new(rpc_router);

	// -- spawn calls
	let mut joinset = JoinSet::new();
	for _ in 0..2 {
		let rpc_router = rpc_router_base.clone();
		let rpc_resources = RpcResourcesBuilder::default().insert(ModelManager).build_owned();
		joinset.spawn(async move {
			let rpc_router = rpc_router.clone();

			let params = json!({"id": 123});

			rpc_router.call("get_task", rpc_resources, Some(params)).await
		});
	}

	// -- print results
	while let Some(result) = joinset.join_next().await {
		println!("res {result:?}");
	}

	Ok(())
}
