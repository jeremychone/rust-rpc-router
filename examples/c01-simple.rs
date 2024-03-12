use rpc_router::{FromResources, Handler, HandlerResult, IntoParams, Request, Resources, Router};
use serde::Deserialize;
use serde_json::json;

#[derive(Clone)]
pub struct ModelManager {}
impl FromResources for ModelManager {}

#[derive(Deserialize)]
pub struct ParamsIded {
	pub id: i64,
}
impl IntoParams for ParamsIded {}

pub async fn increment_id(_mm: ModelManager, params: ParamsIded) -> HandlerResult<i64> {
	Ok(params.id + 9000)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// -- Build the Router with the builder
	let rpc_router = Router::builder()
		// Minor optimization over `.append(...)` to avoid monomorphization
		.append_dyn("increment_id", increment_id.into_dyn())
		.build();

	// -- Build the reqeust
	let rpc_request: Request = json!({
		"jsonrpc": "2.0",
		"id": null, // the json rpc id, that will get echoed back, can be null
		"method": "increment_id",
		"params": {
			"id": 123
		}
	})
	.try_into()?;

	// -- Build the Resources for this call via the builer
	let rpc_resources = Resources::builder().append(ModelManager {}).build();

	// -- Execute
	let call_result = rpc_router.call_with_resources(rpc_request, rpc_resources).await;

	// -- Display result
	match call_result {
		Ok(call_response) => println!(
			"Success: rpc-id {:?}, method: {}, returned value: {:?}",
			call_response.id, call_response.method, call_response.value
		),
		Err(call_error) => println!(
			"Error: rpc-id {:?}, method: {}, error {:?}",
			call_error.id, call_error.method, call_error.error
		),
		// To extract app error type, see code below (examples/c00-readme.md)
	}

	Ok(())
}
