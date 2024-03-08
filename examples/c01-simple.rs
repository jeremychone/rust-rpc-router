pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.

use rpc_router::{FromRpcResources, IntoRpcParams, RpcHandler, RpcResources, RpcResourcesBuilder, RpcRouter};
use serde::Deserialize;
use serde_json::json;

#[derive(Clone)]
pub struct ModelManager;

impl FromRpcResources<RpcResources> for ModelManager {
	fn from_resources(rpc_resources: &RpcResources) -> rpc_router::FromResourcesResult<Self> {
		Ok(rpc_resources.get::<ModelManager>().unwrap())
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

	let mut rpc_router: RpcRouter<RpcResources> = RpcRouter::new();

	rpc_router = rpc_router.add_dyn("get_task", get_task.into_dyn());

	let rr = RpcResourcesBuilder::default().insert(ModelManager).build();

	let res = rpc_router.call("get_task", rr, json!({"id": 123}).try_into()?).await;

	println!("->> {res:?}");

	Ok(())
}
