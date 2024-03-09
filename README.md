# rpc-router - JSON-RPC routing support library

`rpc-router` is a [JSON-RPC](https://www.jsonrpc.org/specification) routing library in Rust for asynchronous dynamic dispatch with support for variadic arguments (up to 8 resources + 1 optional parameter).

The goal of this library is to enable application functions as follows:

(code snippets from: [examples/c00-readme.rs](examples/c00-readme.rs))
```rust
pub async fn create_task(mm: ModelManager, aim: AiManager, params: TaskForCreate) -> Result<i64, MyError> {
  // ...
}
pub async fn get_task(mm: ModelManager, params: ParamsIded) -> Result<Task, MyError> {
  // ...
}
```

With different argument types and signatures, and to be able to call it like this:

```rust
let rpc_router    = router_builder![create_task, get_task].build();
let rpc_resources = resources_builder![ModelManager {..}, AiManager {..}].build();

let rpc_request: rpc_router::Request = json!({
    "jsonrpc": "2.0",
    "id": "some-client-req-id", // json-rpc request id. Can be null,num,string, but has to be present.
    "method": "create_task",
    "params": { "title": "First task" }
}).try_into()?;

let call_response = rpc_router.call(rpc_resources, rpc_request).await?;
let CallResponse { id, method, value } = call_response;
println!(
	r#"RPC call response:
		
    id:  {id:?},
method:  {method},
 value:  {value:?},
"#
	);
```

See [examples/c00-readme.rs](examples/c00-readme.rs) for the complete functioning code.

For the above to work, here are the requirements for the various types:

- `ModelManager` and `AiManager` are rpc-router **Resources**. These types just need to implement `rpc_router::FromResources` (the trait has a default implementation, and `RpcResource` derive macros can generate this one-liner implementation).

```rust
// Make it a Resource with RpcResource derive macro
#[derive(Clone, RpcResource)]
pub struct ModelManager {}

// Make it a Resource by implementing FromResources
#[derive(Clone)]
pub struct AiManager {}
impl FromResources for AiManager {}
```

- `TaskForCreate` and `ParamIded` must implement the `rpc_router::IntoParams` trait, which has a default implementation, and can also be implemented by `RpcParams` derive macros.

```rust
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
```

- `MyError` must implement the `IntoHandlerError`, which also has a default implementation, and can also be implemented by `RpcHandlerError` derive macros.

```rust
#[derive(Debug, thiserror::Error, RpcHandlerError)]
pub enum MyError {
	// TBC
	#[error("TitleCannotBeEmpty")]
	TitleCannotBeEmpty,
}
```

By the Rust type model, these application errors are set in the `HandlerError` and need to be retrieved by `handler_error.get::<MyError>()`. See [examples/c06-error-handling.rs](examples/c06-error-handling.rs).

Full code [examples/c00-readme.rs](examples/c00-readme.rs) 

> **IMPORTANT**
>
> For the `0.1.x` releases, there may be some changes to types or API naming. Therefore, the version should be locked to `=0.1.0`.
>
> Once `0.2.0` is released, I will adhere more strictly to the semantic versioning methodology.

## Concepts

This library has the following main constructs:

1) `Router` - Router is the construct that holds all of the Handler Functions, and can be invoked with `router.call(resources, rpc_request)`. Here are the two main ways to build a `Router` object:
    - **RouterBuilder** - via `RouterBuilder::default()` or `Router::build()`, then call `.add(name, function)` or `.add_dyn(name, function.into_dyn())` to avoid type monomorphization at the "add" stage.
    - **router_builder!** - via the macro `router_builder!(function1, function2, ...)`. This will create, initialize, and return a `RouterBuilder` object.
    - In both cases, call `.build()` to construct the immutable, shareable (via inner Arc) `Router` object.

2) `Resources` - Resources is the type map contstruct that hold the resources that a rpc handler function might request. 
    - It's similar to Axum State/RequestExtractor, or Tauri State model. In the case of `rpc-router` there in one "domain space" for those states, that are called resources. 
    - It's built via `ResourcesBuilder::default().insert(my_object)...build()` or `resources_builder![...].build()`
    - The `Resources` hold the "type map" in a `Arc<>` and is completely immutable and can be cloned effectively. 
	- `ResourcesBuilder` is not wrapped in `Arc<>`, and cloning it will clone the full type map. This can be very useful for sharing a common base resources builder across various calls while allowing each call to add more per-request resources.
    - All the value/object inserted in the Resources must implement `Clone + Send + Sync + 'static` (in this context the `'static` just means that the type cannot have any reference other than static one )

3) `Request` - Is the object that have the json-rpc Request `id`, `method`, and `params`. 
    - To make a struct a `params` it has to implement the `rpc_router::IntoParams` trait, which has the default implementation. 
    - So, simple `impl rpc_router::IntoParams for ... {}`

4) `Handler` - RPC handler functions can be any async application function that takes up to 8 resource arguments, plus an optional Params argument.
    - For example, `async fn create_task(_mm: ModelManager, aim: AiManager, params: TaskForCreate) -> MyResult<i64>`

5) `HandlerError` - RPC handler functions can return their own `Result` as long as the error type implements `IntoHandlerError`, which can be easily implemented as `rpc_router::HandlerResult` which includes an `impl IntoHandlerError for MyError {}`, or with the `RpcHandlerError` derive macro.
    - To allow handler functions to return their application error, `HandlerError` is essentially a type holder, which then allows the extraction of the application error with `handler_error.get<MyError>()`.
    - This requires the application code to know which error type to extract but provides flexibility to return any Error type.
    - Typically, an application will have a few application error types for its handlers, so this ergonomic trade-off still has net positive value as it enables the use of application-specific error types.

6) `CallResult` - `router.call(...)` will return a `CallResult`, which is a `Result<CallResponse, CallError>` where both will include the JSON-RPC `id` and `method` name context for future processing.
    - `CallError` contains `.error: rpc_router::Error`, which includes `rpc_router::Error::Handler(HandlerError)` in the event of a handler error.
    - `CallResponse` contains `.value: serde_json::Value`, which is the serialized value returned by a successful handler call.


## Derive Macros

`rpc-router` has some convenient derive proc macros that generate the implementation of various traits.

This is just a stylistic convenience, as the traits themselves have default implementations and are typically one-liner implementations.

> Note: Those derive proc macros are prefixed with `Rpc` as we often tend to just put the proc macro name in the derive, and therefore the prefix adds some clarity. Other `rpc-router` types, are without the prefix to follow Rust customs.

### `#[derive(rpc_router::RpcParams)]`

Will implement `rpc_router::IntoParams` for the type. 

Works on simple type. 

```rust
#[derive(serde::Deserialize, rpc_router::RpcParams)]
pub strut ParamsIded {
    id: i64
}

// Will generate:
// impl rpc_router::IntoParams for ParamsIded {} 
```

Works with typed with generic (all will be bound to `DeserializeOwned + Send`)

```rust
#[derive(rpc_router::RpcParams)]
pub strut ParamsForUpdate<D> {
    id: i64
    D
}
// Will generate
// impl<D> IntoParams for ParamsForCreate<D> where D: DeserializeOwned + Send {}
```

### `#[derive(rpc_router::RpcResource)]`

Will implement the `rpc_router::FromResource` trait.

```rust

#[derive(Clone, rpc_router::RpcResource)]
pub struct ModelManager;

// Will generate:
// impl FromResources for ModelManager {}
```

The `FromResources` trait has a default implementation to get the `T` type (here `ModelManager`) from the `rpc_router::Resources` type map.

### `#[derive(rpc_router::RpcHandlerError)]`

Will implment the `rpc_router::IntoHandlerError` trait. 

```rust
#[derive(Debug, Serialize, rpc_router::RpcHandlerError)]
pub enum MyError {
    InvalidName,
    // ...
}

// Will generate;
// impl IntoHandlerError for MyError {}
```

<br />

[GitHub Repo](https://github.com/jeremychone/rust-rpc-router)
[crates-io](https://crates.io/crates/rpc-router)