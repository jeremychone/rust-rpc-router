use rpc_router::{router_builder, FromResources, Handler, IntoHandlerError, IntoParams, Resources, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::task::JoinSet;

// region:    --- Custom Error

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

pub async fn get_task(_mm: ModelManager, params: ParamsIded) -> Result<i64, MyError> {
	if params.id > 200 {
		Err(MyError::IdTooBig)
	} else {
		Ok(params.id + 5000)
	}
}

pub async fn get_count(_mm: ModelManager, _params: ParamsIded) -> Result<i64, String> {
	Err("Always a String error".to_string())
}

pub async fn get_count_str(_mm: ModelManager, _params: ParamsIded) -> Result<i64, &'static str> {
	Err("Always a str error")
}

// endregion: --- Test Assets

#[tokio::test]
async fn test_custom_my_error() -> Result<(), Box<dyn std::error::Error>> {
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

			rpc_router
				.call_route_with_resources(None, "get_task", Some(params), rpc_resources)
				.await
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
						format!("HandlerError should be holding a MyError, but was holding {type_name}")
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

#[tokio::test]
async fn test_custom_string_error() -> Result<(), Box<dyn std::error::Error>> {
	let rpc_router = router_builder![
		handlers: [get_task, get_count],
		resources: [ModelManager]
	]
	.build();

	let Err(call_err) = rpc_router.call_route(None, "get_count", Some(json! ({"id": 123}))).await else {
		return Err("Should have returned Error".into());
	};

	let rpc_router::Error::Handler(mut handler_error) = call_err.error else {
		return Err("Should have returned a HandlerError".into());
	};

	assert_eq!(
		handler_error.remove::<String>(),
		Some("Always a String error".to_string())
	);

	Ok(())
}

#[tokio::test]
async fn test_custom_str_error() -> Result<(), Box<dyn std::error::Error>> {
	let rpc_router = router_builder![
		handlers: [get_task, get_count, get_count_str],
		resources: [ModelManager]
	]
	.build();

	let Err(call_err) = rpc_router.call_route(None, "get_count_str", Some(json! ({"id": 123}))).await else {
		return Err("Should have returned Error".into());
	};

	let rpc_router::Error::Handler(mut handler_error) = call_err.error else {
		return Err("Should have returned a HandlerError".into());
	};

	assert_eq!(handler_error.remove::<&'static str>(), Some("Always a str error"));

	Ok(())
}
