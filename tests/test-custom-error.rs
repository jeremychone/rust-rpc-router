pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.

use rpc_router::{FromResources, Handler, IntoHandlerError, IntoParams, Resources, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::task::JoinSet;

// region:    --- Custom Error

pub type MyResult<T> = core::result::Result<T, MyError>;

#[derive(Debug, Serialize)]
pub enum MyError {
	IdTooBig,
	AnotherVariant(String),
}
impl IntoHandlerError for MyError {}

impl core::fmt::Display for MyError {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for MyError {}

// endregion: --- Custom Error

// region:    --- Test Assets

#[derive(Clone)]
pub struct ModelManager;
impl FromResources for ModelManager {}

#[derive(Deserialize)]
pub struct ParamsIded {
	pub id: i64,
}
impl IntoParams for ParamsIded {}

pub async fn get_task(_mm: ModelManager, params: ParamsIded) -> MyResult<i64> {
	if params.id > 200 {
		Err(MyError::IdTooBig)
	} else {
		Ok(params.id + 5000)
	}
}

// endregion: --- Test Assets

#[tokio::test]
async fn test_custom_error_ok() -> Result<()> {
	// -- Setup & Fixtures
	let fx_nums = [123, 222];
	let fx_res_values = [5123, -1];
	let rpc_router = Router::builder().append_dyn("get_task", get_task.into_dyn()).build();
	let rpc_resources = Resources::builder().append(ModelManager).build();

	// -- spawn calls
	let mut joinset = JoinSet::new();
	for (idx, fx_num) in fx_nums.into_iter().enumerate() {
		let rpc_router = rpc_router.clone();
		let rpc_resources = rpc_resources.clone();
		joinset.spawn(async move {
			let rpc_router = rpc_router.clone();

			// Cheap way to "ensure" start spawns matches join_next order. (not for prod)
			tokio::time::sleep(std::time::Duration::from_millis(idx as u64 * 10)).await;

			let params = json!({"id": fx_num});

			rpc_router.call_route(rpc_resources, None, "get_task", Some(params)).await
		});
	}

	// -- Check
	// first, should be 5123
	let res = joinset.join_next().await.ok_or("missing first result")???;
	let res_value: i32 = serde_json::from_value(res.value)?;
	assert_eq!(fx_res_values[0], res_value);

	// second, should be the IdToBig error
	if let Err(err) = joinset.join_next().await.ok_or("missing second result")?? {
		match err.error {
			rpc_router::Error::Handler(handler_error) => {
				if let Some(my_error) = handler_error.get::<MyError>() {
					assert!(
						matches!(my_error, MyError::IdTooBig),
						"should have matched MyError::IdTooBig"
					)
				} else {
					let type_name = handler_error.type_name();
					return Err(
						format!("RpcHandlerError should be holding a MyError, but was holding {type_name}")
							.to_string()
							.into(),
					);
				}
			}
			_other => return Err("second result should be a rpc_router::Error:Handler".to_string().into()),
		}
	} else {
		return Err("second set should have returned error".into());
	}

	Ok(())
}
