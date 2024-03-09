pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.

use rpc_router::{FromRpcResources, IntoRpcParams, RpcHandler, RpcHandlerResult, RpcResourcesBuilder, RpcRouter};
use serde::Deserialize;
use serde_json::json;

#[derive(Clone)]
pub struct ModelManager;
impl FromRpcResources for ModelManager {}

#[derive(Deserialize)]
pub struct ParamsIded {
	pub id: i64,
}
impl IntoRpcParams for ParamsIded {}

pub async fn get_task(_mm: ModelManager, params: ParamsIded) -> RpcHandlerResult<i64> {
	Ok(params.id + 9000)
}

#[tokio::main]
async fn main() -> Result<()> {
	let mut rpc_router: RpcRouter = RpcRouter::new();

	rpc_router = rpc_router.add_dyn("get_task", get_task.into_dyn());

	let rpc_resources = RpcResourcesBuilder::default().insert(ModelManager).build();

	let res = rpc_router.call("get_task", rpc_resources, json!({"id": 123}).try_into()?).await;

	println!("res: {res:?}");

	Ok(())
}
