pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.

use rpc_router::{HandlerResult, RpcParams, RpcResource, router_builder};
use serde::Deserialize;
use serde_json::json;
use tokio::task::JoinSet;

// region:    --- Test Assets

#[derive(Clone, RpcResource)]
pub struct ModelManager;

#[derive(Clone, RpcResource)]
pub struct AiManager;

#[derive(Deserialize, RpcParams)]
pub struct ParamsIded {
	pub id: i64,
}

pub async fn get_task(_mm: ModelManager, _aim: AiManager, params: ParamsIded) -> HandlerResult<i64> {
	Ok(params.id + 9000)
}

// endregion: --- Test Assets

#[tokio::test]
async fn test_base_resources() -> Result<()> {
	// -- Setup & Fixtures
	let fx_num = 125;
	let fx_res_value = 9125;
	let rpc_router = router_builder!(
			handlers: [get_task],
			resources: [ModelManager, AiManager]
	)
	.build();

	// -- spawn calls
	let mut joinset = JoinSet::new();
	for _ in 0..2 {
		let rpc_router = rpc_router.clone();
		joinset.spawn(async move {
			let rpc_router = rpc_router.clone();

			let params = json!({"id": fx_num});

			rpc_router.call_route(None, "get_task", Some(params)).await
		});
	}

	// -- Check
	while let Some(res) = joinset.join_next().await {
		let res = res??;
		let res_value: i32 = serde_json::from_value(res.value)?;
		assert_eq!(res_value, fx_res_value);
	}

	Ok(())
}
