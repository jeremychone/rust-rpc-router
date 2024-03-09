pub type Result<T> = core::result::Result<T, HandlerError>;

use rpc_router::{FromResources, Handler, HandlerError, IntoParams, Resources, Router};
use serde::Deserialize;
use serde_json::json;
use tokio::task::JoinSet;

#[derive(Clone)]
pub struct ModelManager;
impl FromResources for ModelManager {}

#[derive(Deserialize)]
pub struct ParamsIded {
	pub id: i64,
}
impl IntoParams for ParamsIded {}

pub async fn get_task(_mm: ModelManager, params: ParamsIded) -> Result<i64> {
	Ok(params.id + 9000)
}

#[tokio::main]
async fn main() -> Result<()> {
	// -- router & resources
	let rpc_router = Router::builder().append_dyn("get_task", get_task.into_dyn()).build();
	let rpc_resources = Resources::builder().append(ModelManager).build();

	// -- spawn calls
	let mut joinset = JoinSet::new();
	for _ in 0..2 {
		let rpc_router = rpc_router.clone();
		let rpc_resources = rpc_resources.clone();
		joinset.spawn(async move {
			let params = json!({"id": 123});

			// Note: Here we call call_route direct, effectively same call as the call, but just return the Result<Value>
			rpc_router.call_route(rpc_resources, None, "get_task", Some(params)).await
		});
	}

	// -- print results
	while let Some(result) = joinset.join_next().await {
		println!("res {result:?}");
	}

	Ok(())
}
