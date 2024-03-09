use serde_json::Value;

#[derive(Debug)]
pub struct CallResponse {
	pub id: Value,
	pub method: String,
	pub value: Value,
}
