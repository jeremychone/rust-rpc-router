pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.

use rpc_router::{Handler, ResourcesBuilder, Router, RpcHandlerError, RpcParams, RpcResource};
// use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::task::JoinSet;

// region:    --- Custom Error

pub type MyResult<T> = core::result::Result<T, MyError>;

#[derive(Debug, RpcHandlerError, Serialize)]
pub enum MyError {
	// TBC
}

impl core::fmt::Display for MyError {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for MyError {}

// endregion: --- Custom Error

#[derive(Clone, RpcResource)]
pub struct ModelManager {}

#[derive(Deserialize, RpcParams)]
pub struct ParamsIded {
	pub id: i64,
}

// impl<D> IntoParams for ParamsForUpdate<D> where D: DeserializeOwned + Send {}

pub async fn get_task(_mm: ModelManager, params: ParamsIded) -> MyResult<i64> {
	Ok(params.id + 9000)
}

#[tokio::main]
async fn main() -> Result<()> {
	// -- router & resources
	let rpc_router = Router::builder().append_dyn("get_task", get_task.into_dyn()).build();
	let rpc_resources = ResourcesBuilder::default().append(ModelManager {}).build();

	// -- spawn calls
	let mut joinset = JoinSet::new();
	for _ in 0..2 {
		let rpc_router = rpc_router.clone();
		let rpc_resources = rpc_resources.clone();
		joinset.spawn(async move {
			let params = json!({"id": 123});

			rpc_router.call_route(rpc_resources, None, "get_task", Some(params)).await
		});
	}

	// -- Print results
	while let Some(result) = joinset.join_next().await {
		println!("res: {result:?}");
	}

	Ok(())
}
