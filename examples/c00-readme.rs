#![allow(unused)] // For examples.

use rpc_router::{
	resources_builder, router_builder, CallResponse, FromResources, IntoParams, Request, Resources, RpcHandlerError,
	RpcParams, RpcResource,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, thiserror::Error, RpcHandlerError)]
pub enum MyError {
	// TBC
	#[error("TitleCannotBeEmpty")]
	TitleCannotBeEmpty,
}

// Make it a Resource with RpcResource derive macro
#[derive(Clone, RpcResource)]
pub struct ModelManager {}

// Make it a Resource by implementing FromResources
#[derive(Clone)]
pub struct AiManager {}
impl FromResources for AiManager {}

// Make it a Params with RpcParams derive macro
#[derive(Serialize, Deserialize, RpcParams)]
pub struct TaskForCreate {
	title: String,
	done: Option<bool>,
}

// Make it a Params by implementing IntoParams
#[derive(Deserialize)]
pub struct ParamsIded {
	pub id: i64,
}
impl IntoParams for ParamsIded {}

// Return values just need to implement Serialize
#[derive(Serialize)]
pub struct Task {
	id: i64,
	title: String,
	done: bool,
}

pub async fn get_task(mm: ModelManager, params: ParamsIded) -> Result<Task, MyError> {
	Ok(Task {
		id: params.id,
		title: "fake task".to_string(),
		done: false,
	})
}

pub async fn create_task(mm: ModelManager, aim: AiManager, params: TaskForCreate) -> Result<i64, MyError> {
	Ok(123) // return fake id
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Build the sharable router.
	let rpc_router = router_builder![get_task, create_task].build();

	// Build the sharable resources ("type map").
	let rpc_resources = resources_builder![ModelManager {}, AiManager {}].build();

	// Create and parse rpc request example.
	let rpc_request: Request = json!({
		"jsonrpc": "2.0",
		"id": "some-client-req-id", // the json rpc id, that will get echoed back, can be null
		"method": "get_task",
		"params": {
			"id": 123
		}
	})
	.try_into()?;

	// Async Execute the RPC Request.
	let call_response = rpc_router.call(rpc_resources, rpc_request).await?;

	// Display the response.
	let CallResponse { id, method, value } = call_response;
	println!(
		r#"RPC call response:

    id:  {id:?},
method:  {method},
 value:  {value:?},
"#
	);

	Ok(())
}
