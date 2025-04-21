pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.

use rpc_router::{FromResources, Handler, HandlerResult, IntoParams, RpcRequest, Resources, Router, RpcId};
use serde::Deserialize;
use serde_json::json;
use tokio::task::JoinSet;

// region:    --- Test Assets

#[derive(Clone)]
pub struct ModelManager;
impl FromResources for ModelManager {}

#[derive(Deserialize)]
pub struct ParamsIded {
	pub id: i64,
}
impl IntoParams for ParamsIded {}

pub async fn get_task(_mm: ModelManager, params: ParamsIded) -> HandlerResult<i64> {
	Ok(params.id + 9000)
}

// endregion: --- Test Assets

#[tokio::test]
async fn test_sync_call() -> Result<()> {
	// -- Setup & Fixtures
	let fx_num = 123;
	let fx_res_value = 9123;
	let rpc_router = Router::builder()
		.append_dyn("get_task", get_task.into_dyn())
		.append_resource(ModelManager)
		.build();

	let rpc_request: RpcRequest = json!({
		"jsonrpc": "2.0",
		"id": null, // the json rpc id, that will get echoed back, can be null
		"method": "get_task",
		"params": {
			"id": fx_num
		}
	})
	.try_into()?;

	// -- Exec
	let res = rpc_router.call(rpc_request).await?;

	// -- Check
	let res_value: i32 = serde_json::from_value(res.value)?;
	assert_eq!(res_value, fx_res_value);

	Ok(())
}

#[tokio::test]
async fn test_async_calls() -> Result<()> {
	// -- Setup & Fixtures
	let fx_num = 124;
	let fx_res_value = 9124;
	let rpc_router = Router::builder().append_dyn("get_task", get_task.into_dyn()).build();

	// -- spawn calls
	let mut joinset = JoinSet::new();
	for idx in 0..2 {
		let rpc_router = rpc_router.clone();
		let rpc_resources = Resources::builder().append(ModelManager).build();
		let rpc_request: RpcRequest = json!({
			"jsonrpc": "2.0",
			"id": idx, // the json rpc id, that will get echoed back, can be null
			"method": "get_task",
			"params": {
				"id": fx_num
			}
		})
		.try_into()?;

		joinset.spawn(async move {
			let rpc_router = rpc_router.clone();

			rpc_router.call_with_resources(rpc_request, rpc_resources).await
		});
	}

	// -- Check
	let mut fx_rpc_id_num = 0;
	while let Some(res) = joinset.join_next().await {
		let rpc_response = res??;

		// check rpc_id
		let fx_rpc_id = RpcId::from_value(json!(fx_rpc_id_num))?;
		assert_eq!(rpc_response.id, fx_rpc_id);
		fx_rpc_id_num += 1;

		// check result value
		let res = rpc_response.value;
		let res_value: i32 = serde_json::from_value(res)?;
		assert_eq!(res_value, fx_res_value);
	}

	Ok(())
}

#[tokio::test]
async fn test_shared_resources() -> Result<()> {
	// -- Setup & Fixtures
	let fx_num = 125;
	let fx_res_value = 9125;
	let rpc_router = Router::builder().append_dyn("get_task", get_task.into_dyn()).build();
	let rpc_resources = Resources::builder().append(ModelManager).build();

	// -- spawn calls
	let mut joinset = JoinSet::new();
	for _ in 0..2 {
		let rpc_router = rpc_router.clone();
		let rpc_resources = rpc_resources.clone();
		joinset.spawn(async move {
			let rpc_router = rpc_router.clone();

			let params = json!({"id": fx_num});

			rpc_router
				.call_route_with_resources(None, "get_task", Some(params), rpc_resources)
				.await
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
