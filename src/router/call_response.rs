use serde_json::Value;

/// The successful response back from a `rpc_router.call...` functions.
///  
/// NOTE: CallResponse & CallError
///       are not designed to be the JSON-RPC Response
///       or Error, but to provide the necessary context
///       to build those, as well as the useful `method name`
///       context for tracing/login.
#[derive(Debug)]
pub struct CallResponse {
	pub id: Value,
	pub method: String,
	pub value: Value,
}
