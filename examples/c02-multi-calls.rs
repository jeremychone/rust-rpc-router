pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.

use rpc_router::{Handler, IntoParams, Resources, Router, RpcRequest, RpcResource};
use serde::Deserialize;
use serde_json::json;
use tokio::task::JoinSet;

#[derive(Clone, RpcResource)]
pub struct ModelManager;

#[derive(Clone, RpcResource)]
pub struct UserCtx {
	_user_id: i64,
}

#[derive(Deserialize)]
pub struct ParamsIded {
	pub id: i64,
}
impl IntoParams for ParamsIded {}

pub async fn get_task(_ctx: UserCtx, _mm: ModelManager, params: ParamsIded) -> rpc_router::HandlerResult<i64> {
	Ok(params.id + 9000)
}

#[tokio::main]
async fn main() -> Result<()> {
	// -- router
	let rpc_router = Router::builder()
		.append_dyn("get_task", get_task.into_dyn())
		.append_resource(ModelManager)
		.build();

	// -- spawn calls
	let mut joinset = JoinSet::new();
	for idx in 0..2 {
		let rpc_router = rpc_router.clone();
		let rpc_request: RpcRequest = json!({
			"jsonrpc": "2.0",
			"id": idx, // the json rpc id, that will get echoed back, can be null
			"method": "get_task",
			"params": {
				"id": 123
			}
		})
		.try_into()?;

		joinset.spawn(async move {
			// Cheap way to "ensure" start spawns matches join_next order. (not for prod)
			tokio::time::sleep(std::time::Duration::from_millis(idx as u64 * 10)).await;

			// Build the additional resources to overlay on top of the router resources
			let addtional_resources = Resources::builder().append(UserCtx { _user_id: 123 }).build();

			// Exec the call
			rpc_router.call_with_resources(rpc_request, addtional_resources).await
		});
	}

	// -- print results
	// Should have id: 0, and then, id: 1
	while let Some(response) = joinset.join_next().await {
		println!("res {response:?}");
	}

	Ok(())
}
