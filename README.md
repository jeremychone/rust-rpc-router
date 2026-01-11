# rpc-router - JSON-RPC Routing Library

API changes for `v0.2.0`
- `RpcId` - Now uses a concrete type for RpcId.
- `RpcRequest` - The old `Request` is now renamed `RpcRequest`. The design is that raw JSON-RPC constructs are prefixed with `Rpc`.
- `RpcNotification` - New type (Like `RpcRequest` but with no `.id` as per the spec).
- `RpcResponse` - New type.

## Getting Started

`rpc-router` is a [JSON-RPC](https://www.jsonrpc.org/specification) routing library in Rust for asynchronous dynamic dispatch with support for variadic arguments (up to 8 resources + 1 optional parameter). (Code snippets below are from: [examples/c00-readme.rs](examples/c00-readme.rs))

The goal of this library is to enable application functions with different argument types and signatures as follows:

```rust
pub async fn create_task(mm: ModelManager, aim: AiManager, params: TaskForCreate) -> Result<i64, MyError> {
  // ...
}
pub async fn get_task(mm: ModelManager, params: ParamsIded) -> Result<Task, MyError> {
  // ...
}
```

To be callable from a JSON-RPC request as follows:

```rust
// JSON-RPC request coming from Axum route payload, Tauri command params, ...
let rpc_request = json!(
   { jsonrpc: "2.0", id: 1,          // required by JSON-RPC
     method:  "create_task",         // method name (matches function name)
     params:  {title: "First Task"}  // optional params (last function argument)
   }).try_into()?;

// Async execute the RPC request 
let call_response = rpc_router.call(rpc_request).await?;
```

For this, we just need to build the router, the resources, parse the JSON-RPC request, and execute the call from the router as follows:

```rust
// Build the Router with the handlers and common resources
let rpc_router = router_builder!(
    handlers: [get_task, create_task],         // will be turned into routes
    resources: [ModelManager {}, AiManager {}] // common resources for all calls
)
.build();
// Can do the same with `Router::builder().append(...)/append_resource(...)`

// Create and parse rpc request example.
let rpc_request: rpc_router::Request = json!({
    "jsonrpc": "2.0",
    "id": "some-client-req-id", // JSON-RPC request id. Can be null, num, string, but must be present.
    "method": "create_task",
    "params": { "title": "First task" } // optional.
}).try_into()?;

// Async execute the RPC request.
let call_response = rpc_router.call(rpc_resources, rpc_request).await?;

// Or `call_with_resources` for additional per-call resources that override router common resources.
// e.g., rpc_router.call_with_resources(rpc_request, additional_resources)

// Display the response.
let CallSuccess { id, method, value } = call_response;
println!(
	r#"RPC call response:
		
    id:  {id:?},
method:  {method},
 value:  {value:?},
"#
	);
```

See [examples/c00-readme.rs](examples/c00-readme.rs) for the complete working code.

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

- `TaskForCreate` and `ParamsIded` are used as JSON-RPC Params and must implement the `rpc_router::IntoParams` trait, which has a default implementation, and can also be implemented by `RpcParams` derive macros.

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

- `Task`, as a returned value, just needs to implement `serde::Serialize`

```rust
#[derive(Serialize)]
pub struct Task {
	id: i64,
	title: String,
	done: bool,
}
```

- `MyError` must implement `IntoHandlerError`, which also has a default implementation, and can also be implemented by `RpcHandlerError` derive macros.

```rust
#[derive(Debug, thiserror::Error, RpcHandlerError)]
pub enum MyError {
	// TBC
	#[error("TitleCannotBeEmpty")]
	TitleCannotBeEmpty,
}
```

By the Rust type model, these application errors are set in the `HandlerError` and need to be retrieved by `handler_error.get::<MyError>()`. See [examples/c05-error-handling.rs](examples/c05-error-handling.rs).

Full code: [examples/c00-readme.rs](examples/c00-readme.rs)

> **IMPORTANT**
>
> For the `0.1.x` releases, there may be some changes to types or API naming. Therefore, the version should be locked to the latest version used, for example, `=0.1.0`. I will try to keep changes to a minimum, if any, and document them in the future [CHANGELOG](CHANGELOG.md).
>
> Once `0.2.0` is released, I will adhere more strictly to semantic versioning.

### Concepts

This library has the following main constructs:

1) `Router` - Router is the construct that holds all of the Handler Functions and can be invoked with `router.call(resources, rpc_request)`. Here are the two main ways to build a `Router` object:
    - **RouterBuilder** - via `RouterBuilder::default()` or `Router::builder()`, then call `.append(name, function)` or `.append_dyn(name, function.into_dyn())` to avoid type monomorphization at the "append" stage.
    - **router_builder!** - via the macro `router_builder!(function1, function2, ...)`. This will create, initialize, and return a `RouterBuilder` object.
    - In both cases, call `.build()` to construct the immutable, shareable (via inner Arc) `Router` object.

2) `Resources` - Resources is the type map construct that holds the resources that an RPC handler function might request. 
    - It's similar to Axum State/RequestExtractor or the Tauri State model. In the case of `rpc-router`, there is one "domain space" for those states called resources. 
    - It's built via `ResourcesBuilder::default().append(my_object)...build()`.
	- Or via the macro `resources_builder![my_object1, my_object2].build()`.
    - The `Resources` hold the type map in an `Arc<>` and are completely immutable and can be cloned effectively. 
	- `ResourcesBuilder` is not wrapped in an `Arc<>`, and cloning it will clone the full type map. This can be very useful for sharing a common base resources builder across various calls while allowing each call to add more per-request resources.
    - All the values/objects inserted into the Resources must implement `Clone + Send + Sync + 'static` (here `'static` means the type cannot have any references other than static ones).

3) `Request` - Is the object that has the JSON-RPC Request `id`, `method`, and `params`. 
    - To make a struct a `params`, it has to implement the `rpc_router::IntoParams` trait, which has a default implementation. 
    - So, implement `impl rpc_router::IntoParams for ... {}` or `#[derive(RpcParams)]`.
	- `rpc_router::Request::from_value(serde_json::Value) -> Result<Request, RequestParsingError>` will return a `RequestParsingError` if the Value does not have `id: Value`, `method: String` or if the Value does not contain `"jsonrpc": "2.0"` as per the JSON-RPC spec. 
	- `let request: rpc_router::Request = value.try_into()?` uses the same `from_value` validation steps.
	- Doing `serde_json::from_value::<rpc_router::Request>(value)` will not change the `jsonrpc`.

4) `Handler` - RPC handler functions can be any async application function that takes up to 8 resource arguments, plus an optional Params argument.
    - For example, `async fn create_task(_mm: ModelManager, aim: AiManager, params: TaskForCreate) -> MyResult<i64>`

5) `HandlerError` - RPC handler functions can return their own `Result` as long as the error type implements `IntoHandlerError`, which can be easily implemented as `rpc_router::HandlerResult` which includes an `impl IntoHandlerError for MyError {}`, or with the `RpcHandlerError` derive macro.
    - To allow handler functions to return their application error, `HandlerError` is essentially a type holder that allows the extraction of the application error with `handler_error.get<MyError>()`.
    - This requires the application code to know which error type to extract but provides flexibility to return any Error type.
    - Typically, an application will have a few application error types for its handlers, so this ergonomic trade-off still has net positive value as it enables the use of application-specific error types.

6) `CallResult` - `router.call(...)` will return a `CallResult`, which is a `Result<CallSuccess, CallError>` where both include the JSON-RPC `id` and `method` name context for future processing.
    - `CallError` contains `.error: rpc_router::Error`, which includes `rpc_router::Error::Handler(HandlerError)` in the event of a handler error.
    - `CallSuccess` contains `.value: serde_json::Value`, which is the serialized value returned by a successful handler call.

### Derive Macros

`rpc-router` has some convenient derive proc macros that generate the implementation of various traits.

This is just a stylistic convenience, as the traits themselves have default implementations and are typically one-liner implementations.

> Note: These derive proc macros are prefixed with `Rpc` since macros often have generic names, so the prefix adds clarity. Other `rpc-router` types are without the prefix to follow Rust customs.

### `#[derive(rpc_router::RpcParams)]`

Implements `rpc_router::IntoParams` for the type. 

Works on simple types.

```rust
#[derive(serde::Deserialize, rpc_router::RpcParams)]
pub struct ParamsIded {
    id: i64
}

// Will generate:
// impl rpc_router::IntoParams for ParamsIded {} 
```

Works with generic types (all will be bound to `DeserializeOwned + Send`):

```rust
#[derive(rpc_router::RpcParams)]
pub struct ParamsForUpdate<D> {
    id: i64,
    D
}
// Will generate
// impl<D> IntoParams for ParamsForUpdate<D> where D: DeserializeOwned + Send {}
```

### `#[derive(rpc_router::RpcResource)]`

Implements the `rpc_router::FromResource` trait.

```rust
#[derive(Clone, rpc_router::RpcResource)]
pub struct ModelManager;

// Will generate:
// impl FromResources for ModelManager {}
```

The `FromResources` trait has a default implementation to get the `T` type (here `ModelManager`) from the `rpc_router::Resources` type map.

### `#[derive(rpc_router::RpcHandlerError)]`

Implements the `rpc_router::IntoHandlerError` trait.

```rust
#[derive(Debug, Serialize, RpcHandlerError)]
pub enum MyError {
    InvalidName,
    // ...
}

// Will generate:
// impl IntoHandlerError for MyError {}
```

<br />

## Related Links

- [GitHub Repo](https://github.com/jeremychone/rust-rpc-router)
- [crates.io](https://crates.io/crates/rpc-router)
- [Rust10x rust-web-app](https://rust10x.com/web-app) (web-app code blueprint using [rpc-router](https://github.com/jeremychone/rust-rpc-router) with [Axum](https://github.com/tokio-rs/axum))