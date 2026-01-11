# RPC Router - All APIs

This document provides a comprehensive overview of all public APIs exposed by the `rpc-router` crate.

## Key Features

- **Type-Safe Handlers:** Define RPC handlers as regular async Rust functions with typed parameters derived from request resources and JSON-RPC parameters.
- **Resource Management:** Inject shared resources (like database connections, configuration, etc.) into handlers using the `FromResources` trait.
- **Parameter Handling:** Automatically deserialize JSON-RPC `params` into Rust types using the `IntoParams` trait. Supports optional parameters and default values via `IntoDefaultRpcParams`.
- **Composable Routers:** Combine multiple routers using `RouterBuilder::extend` for modular application design.
- **Error Handling:** Robust error handling with distinct types for routing errors (`rpc_router::Error`) and handler-specific errors (`HandlerError`), allowing applications to retain and inspect original error types.
- **JSON-RPC Response Types:** Provides `RpcResponse`, `RpcError`, and `RpcResponseParsingError` for representing and parsing standard JSON-RPC 2.0 responses, with direct conversion from router `CallResult`.
- **Ergonomic Macros (Optional):** Includes helper macros like `router_builder!` and `resources_builder!` for concise router and resource setup.
- **Minimal Dependencies:** Core library has minimal dependencies (primarily `serde`, `serde_json`, and `futures`).

## Core Types

-   **`Router`**: The central component that holds RPC method routes and associated handlers. It's typically created using `RouterBuilder` and wrapped in an `Arc` for efficient sharing.
-   **`RouterBuilder`**: Used to configure and build a `Router`. Allows adding handlers and base resources.
-   **`Resources`**: A type map used to hold shared application state or resources (like database connections, configuration, etc.) accessible by handlers.
-   **`ResourcesBuilder`**: Used to configure and build `Resources`.

## Request Handling Flow

1.  **Parsing the Message**: Incoming JSON messages are parsed.
    -   If the message contains an `id`, it's parsed into an `RpcRequest`.
    -   If the message lacks an `id`, it's parsed into an `RpcNotification`.
    -   Parsing involves strict validation against the JSON-RPC 2.0 specification (e.g., `jsonrpc: "2.0"`, method format, params format).
2.  **Calling the Router (for Requests)**: The `Router::call` or `Router::call_with_resources` method is invoked with the `RpcRequest` and optional additional `Resources`.
3.  **Method Routing**: The router finds the handler registered for the requested `method`.
4.  **Resource Injection**: The router attempts to extract required resources (types implementing `FromResources`) from the provided `Resources`.
5.  **Parameter Deserialization**: If the handler expects parameters, the `params` field of the `RpcRequest` (an `Option<Value>`) is deserialized into the expected type using the `IntoParams` trait.
6.  **Handler Execution**: The asynchronous handler function is called with the injected resources and deserialized parameters.
7.  **Result Handling**: The handler returns a `HandlerResult<T>` (which is `Result<T, HandlerError>`).
8.  **Router Response**: The router captures the handler's result or any errors during resource/parameter handling and wraps it in a `CallResult` (`Result<CallSuccess, CallError>`).
9.  **JSON-RPC Response Formation**: The `CallResult` is typically converted into a standard JSON-RPC `RpcResponse` (containing either `RpcSuccessResponse` or `RpcErrorResponse`) for sending back to the client. This conversion handles mapping internal router errors (`router::Error`) to standard `RpcError` objects.
10. **Handling Notifications**: Notifications (`RpcNotification`) are typically processed separately from requests, as they do not produce a response. The application logic decides how to handle them (e.g., logging, triggering background tasks). The `Router` itself is primarily designed for request-response interactions.

## JSON-RPC Message Parsing

### `RpcRequest` (Request with ID)

-   **`RpcRequest`**: Represents a parsed JSON-RPC request that expects a response.
    ```rust
    pub struct RpcRequest {
        pub id: RpcId,
        pub method: String,
        pub params: Option<Value>, // Must be Object or Array if Some
    }
    ```
-   **`RpcId`**: An enum representing the JSON-RPC ID (`String(Arc<str>)`, `Number(i64)`, or `Null`). See `RpcId` section below for details.
-   **Parsing**:
    -   **`RpcRequest::from_value(value: Value) -> Result<RpcRequest, RpcRequestParsingError>`**: Parses a `serde_json::Value` into an `RpcRequest`. Performs strict validation (`RpcRequestCheckFlags::ALL`: `VERSION` and `ID`).
    -   **`RpcRequest::from_value_with_checks(value: Value, checks: RpcRequestCheckFlags) -> Result<RpcRequest, RpcRequestParsingError>`**: Parses with customizable validation.
        -   `RpcRequestCheckFlags`: Bitflags to control checks:
            -   `VERSION`: Checks for `"jsonrpc": "2.0"`.
            -   `ID`: Checks for presence and validity (String, Number, *not* Null) of the `id` field. If flag is *not* set, defaults to `RpcId::Null` if `id` is missing or invalid.
            -   `ALL`: Enables `VERSION` and `ID`.
        -   Example (skip ID check, allowing missing ID):
            ```rust
            use rpc_router::{RpcRequest, RpcRequestCheckFlags, RpcRequestParsingError, RpcId};
            use serde_json::json;

            let request_value = json!({
              "jsonrpc": "2.0",
              // "id": 123, // ID missing, but we'll skip the check
              "method": "my_method",
              "params": [1, 2]
            });

            // Only check version
            let flags = RpcRequestCheckFlags::VERSION;
            let result = RpcRequest::from_value_with_checks(request_value, flags);

            assert!(result.is_ok());
            let request = result.unwrap();
            // request.id defaults to RpcId::Null when check is skipped and id is missing
            assert_eq!(request.id, RpcId::Null);
            assert_eq!(request.method, "my_method");
            ```
    -   **`TryFrom<Value> for RpcRequest`**: Convenience trait implementation calling `RpcRequest::from_value_with_checks(value, RpcRequestCheckFlags::ALL)`.
    -   **`RpcRequestParsingError`**: Enum detailing specific parsing failures (e.g., `VersionMissing`, `IdMissing`, `MethodInvalidType`, `ParamsInvalidType`).

### `RpcNotification` (Request without ID)

-   **`RpcNotification`**: Represents a parsed JSON-RPC notification (request without an `id`). No response is generated for notifications.
    ```rust
    pub struct RpcNotification {
        pub method: String,
        pub params: Option<Value>, // Must be Object or Array if Some
    }
    ```
-   **Parsing**:
    -   **`RpcNotification::from_value(value: Value) -> Result<RpcNotification, RpcRequestParsingError>`**: Parses a `serde_json::Value` into an `RpcNotification`. Performs strict validation: checks `"jsonrpc": "2.0"`, presence/type of `method`, type of `params`, and *absence* of `id`.
    -   **`TryFrom<Value> for RpcNotification`**: Convenience trait implementation calling `RpcNotification::from_value`.
    -   **`RpcRequestParsingError`**: Shared error enum with `RpcRequest`. Relevant variants include `NotificationHasId`, `VersionMissing`, `MethodMissing`, `ParamsInvalidType` etc.

### `RpcId`

-   **`RpcId`**: Enum representing the JSON-RPC ID.
    ```rust
    pub enum RpcId {
        String(Arc<str>),
        Number(i64),
        Null,
    }
    ```
-   Implements `Serialize`, `Deserialize`, `Clone`, `Debug`, `PartialEq`, `Eq`, `Hash`, `Display`, `Default` (defaults to `Null`).
-   **Constructors**:
    -   `RpcId::from_scheme(kind: IdSchemeKind, enc: IdSchemeEncoding) -> Self`: Generates a new String ID based on a scheme (e.g., UUID v4/v7) and encoding (e.g., standard, Base64, Base58).
    -   Convenience constructors: `new_uuid_v4()`, `new_uuid_v4_base64()`, `new_uuid_v4_base64url()`, `new_uuid_v4_base58()`, `new_uuid_v7()`, `new_uuid_v7_base64()`, `new_uuid_v7_base64url()`, `new_uuid_v7_base58()`.
-   **Conversion**:
    -   `From<String>`, `From<&str>`, `From<i64>`, `From<i32>`, `From<u32>`.
    -   `to_value(&self) -> Value`.
    -   `from_value(value: Value) -> Result<Self, RpcRequestParsingError>`: Parses from `Value`, returning `RpcRequestParsingError::IdInvalid` on failure.

## Router Invocation (`Router` Methods)

-   **`Router::call(&self, rpc_request: RpcRequest) -> impl Future<Output = CallResult>`**: Executes the request using the router's base resources.
-   **`Router::call_with_resources(&self, rpc_request: RpcRequest, additional_resources: Resources) -> impl Future<Output = CallResult>`**: Executes the request, overlaying `additional_resources` on top of the base resources. Resources are looked up first in `additional_resources`, then in the base resources.
-   **`Router::call_route(&self, id: Option<RpcId>, method: impl Into<String>, params: Option<Value>) -> impl Future<Output = CallResult>`**: Lower-level call using individual components instead of `RpcRequest`. Uses base resources. `id` defaults to `RpcId::Null` if `None`.
-   **`Router::call_route_with_resources(...)`**: Like `call_route` but with `additional_resources`.

## Router Call Output (`CallResult`)

-   **`CallResult`**: Type alias for `Result<CallSuccess, CallError>`. This is the return type of the `Router::call...` methods. This is an *intermediate* result used internally and to provide context (like method name) for logging or tracing before creating the final `RpcResponse`.
-   **`CallSuccess`**: Struct containing the successful result details.
    ```rust
    pub struct CallSuccess {
        pub id: RpcId,     // The ID from the original RpcRequest
        pub method: String, // The method from the original RpcRequest
        pub value: Value,  // Serialized result from the handler
    }
    ```
-   **`CallError`**: Struct containing error details.
    ```rust
    pub struct CallError {
        pub id: RpcId,     // The ID from the original RpcRequest
        pub method: String, // The method from the original RpcRequest
        pub error: router::Error, // The router/handler error (see Error Handling)
    }
    ```
    Implements `std::error::Error`, `Display`, `Debug`.

## Defining Handlers (`Handler` Trait)

-   **Signature**: Handlers are `async` functions with the following general signature:
    ```rust
    async fn handler_name(
        [resource1: T1, resource2: T2, ...] // 0 or more resources implementing FromResources
        [params: P]                         // 0 or 1 parameter implementing IntoParams
    ) -> HandlerResult<R>                   // R must implement Serialize
    where
        T1: FromResources, T2: FromResources, ...,
        P: IntoParams,
        R: Serialize
    {
        // ... logic ...
        Ok(result_value) // or Err(handler_error)
    }
    ```
-   **`HandlerResult<T>`**: Alias for `Result<T, HandlerError>`. Handlers should return this.
-   **`Handler` Trait**: Automatically implemented for functions matching the required signatures (up to 8 resource parameters). You generally don't interact with this trait directly.
    -   `fn call(self, resources: Resources, params: Option<Value>) -> PinFutureValue;`
    -   `fn into_dyn(self) -> Box<dyn RpcHandlerWrapperTrait>;`: Used internally by `RouterBuilder` macros/methods.

### Handler Parameters (`IntoParams`)

-   **`IntoParams` Trait**: Implement this for types you want to use as the `params` argument in your handlers.
    -   Requires `DeserializeOwned + Send`.
    -   Default `into_params(value: Option<Value>) -> Result<Self>` method deserializes from `Some(Value)`, returns `Error::ParamsMissingButRequested` if `None`.
-   **`IntoDefaultRpcParams` Trait**: Marker trait. If a type implements `IntoDefaultRpcParams` and `Default`, `IntoParams` is automatically implemented such that `T::default()` is used when JSON-RPC params are `null` or absent (`None`).
-   **Derive Macro `#[derive(RpcParams)]`** (requires `rpc-router-macros` feature): The recommended way to implement `IntoParams` for simple structs. Equivalent to `impl IntoParams for MyType {}`.
-   **Derive Macro `#[derive(RpcParams, Default)]`** (requires `rpc-router-macros` feature): The recommended way to implement `IntoDefaultRpcParams`. Equivalent to `impl IntoDefaultRpcParams for MyType {}` (assuming `Default` is also derived or implemented).
-   **Blanket Impls (Optional Features - Use with caution)**:
    -   `Option<T>`: `IntoParams` is implemented for `Option<T>` where `T: IntoParams`. Allows optional parameters.
    -   `Value`: `IntoParams` is implemented directly for `serde_json::Value`. Avoids strong typing.

### Handler Resources (`FromResources`)

-   **`FromResources` Trait**: Implement this for types you want to inject as resource arguments into your handlers.
    -   Requires the type to be `Clone + Send + Sync + 'static`.
    -   Default `from_resources(resources: &Resources) -> FromResourcesResult<Self>` method retrieves the type from `Resources`. Returns `FromResourcesError::ResourceNotFound` if not found.
-   **Derive Macro `#[derive(RpcResource)]`** (requires `rpc-router-macros` feature): Recommended way. Equivalent to `impl FromResources for MyType {}`. Ensure the type also implements `Clone + Send + Sync + 'static`.
-   **Blanket Impl `Option<T>`**: `FromResources` is implemented for `Option<T>` where `T: FromResources`, allowing optional resource injection (returns `Ok(None)` if the resource `T` is not found).

## Resources Management (`Resources`, `ResourcesBuilder`)

-   **`Resources`**: A cloneable type map (`Arc`-based internally for cheap cloning). Holds shared state accessible by handlers.
    -   `Resources::builder() -> ResourcesBuilder`
    -   `get<T: Clone + Send + Sync + 'static>(&self) -> Option<T>`: Retrieves a clone of the resource. Looks in overlay resources first, then base resources.
    -   `is_empty(&self) -> bool`: Checks if both base and overlay resources are empty.
-   **`ResourcesBuilder`**: Used to construct `Resources`.
    -   `default()`
    -   `append<T: Clone + Send + Sync + 'static>(self, val: T) -> Self`: Adds a resource.
    -   `append_mut<T>(&mut self, val: T)`: Adds a resource without consuming the builder.
    -   `get<T: Clone + Send + Sync + 'static>(&self) -> Option<T>`: Gets a resource from the builder (before building).
    -   `build(self) -> Resources`: Creates the `Resources` object (builds the base resources).
-   **`resources_builder!` Macro**: Convenience for creating a `ResourcesBuilder` and appending items.
    ```rust
    use rpc_router::{resources_builder, Resources, FromResources};
    use std::sync::Arc;

    #[derive(Clone)] struct MyDb;
    impl FromResources for MyDb {} // Required for Resources

    #[derive(Clone)] struct MyConfig;
    impl FromResources for MyConfig {} // Required for Resources

    let resources: Resources = resources_builder!(
        Arc::new(MyDb{}), // Resources typically need to be Arc'd or cheap to clone
        Arc::new(MyConfig{})
    ).build();
    ```

## Router Configuration (`RouterBuilder`)

-   **`RouterBuilder::default()`**: Creates a new builder.
-   **`append<F, T, P, R>(self, name: &'static str, handler: F) -> Self`**: Adds a handler function directly by name. Infers types, convenient but causes monomorphization for each handler signature.
    ```rust
    use rpc_router::{RouterBuilder, HandlerResult};
    async fn my_async_handler() -> HandlerResult<String> { Ok("hello".to_string()) }
    let builder = RouterBuilder::default().append("my_method", my_async_handler);
    ```
-   **`append_dyn(self, name: &'static str, dyn_handler: Box<dyn RpcHandlerWrapperTrait>) -> Self`**: Adds a type-erased handler. Preferred for dynamic route addition or avoiding monomorphization bloat. Requires manual `into_dyn()` call.
    ```rust
    use rpc_router::{RouterBuilder, Handler, HandlerResult}; // For .into_dyn() trait method
    async fn my_async_handler() -> HandlerResult<String> { Ok("hello".to_string()) }
    let builder = RouterBuilder::default().append_dyn("my_method", my_async_handler.into_dyn());
    ```
-   **`append_resource<T>(self, val: T) -> Self`**: Adds a base resource available to all handlers invoked through the resulting `Router`. Requires `T: FromResources + Clone + Send + Sync + 'static`.
-   **`extend(self, other_builder: RouterBuilder) -> Self`**: Merges another builder's routes and base resources into this one. Routes/resources from `other_builder` overwrite existing ones with the same name/type.
-   **`extend_resources(self, resources_builder: Option<ResourcesBuilder>) -> Self`**: Adds base resources from another `ResourcesBuilder`. If `self` already has resources, they are merged (new ones overwrite old).
-   **`set_resources(self, resources_builder: ResourcesBuilder) -> Self`**: Replaces the router's base resources entirely with those from the provided builder.
-   **`build(self) -> Router`**: Builds the final `Router`. The `Router` is cloneable (`Arc`-based).
-   **`router_builder!` Macro**: Convenience macro for building a `RouterBuilder`.
    ```rust
    use rpc_router::{router_builder, Router, Handler, HandlerError, FromResources};

    async fn handler_one() -> Result<String, HandlerError> { Ok("one".to_string()) }
    async fn handler_two() -> Result<String, HandlerError> { Ok("two".to_string()) }

    #[derive(Clone)] struct DbRes;
    impl FromResources for DbRes {}

    // Pattern 1: Simple list of handlers
    let router1: Router = router_builder!(handler_one, handler_two).build();

    // Pattern 2: Explicit handlers and resources lists
    let router2: Router = router_builder!(
        handlers: [handler_one, handler_two],
        resources: [DbRes{}]
    ).build();

    // Pattern 3: Explicit handlers list only
    let router3: Router = router_builder!(
        handlers: [handler_one]
    ).build();
    ```

## Error Handling

-   **`router::Error`**: Enum representing errors occurring *within* the router logic or during handler invocation setup (parameter parsing, resource fetching). This is the error type within `CallResult::Err(CallError)`.
    ```rust
    pub enum Error {
        // -- Into Params Errors
        ParamsParsing(serde_json::Error),
        ParamsMissingButRequested,

        // -- Router Error
        MethodUnknown,

        // -- Handler Setup/Execution Errors
        FromResources(FromResourcesError),
        HandlerResultSerialize(serde_json::Error), // Error serializing the handler's Ok(value)
        Handler(HandlerError), // Wrapper for an error returned *by* the handler itself (Err(handler_error))
    }
    ```
    Implements `Debug`, `Serialize`, `Display`, `std::error::Error`. `From<HandlerError>`, `From<FromResourcesError>`.
-   **`FromResourcesError`**: Error specifically from failing to get a resource via `FromResources`.
    ```rust
    pub enum FromResourcesError {
        ResourceNotFound(&'static str), // Contains the name of the type not found.
    }
    ```
    Implements `Debug`, `Serialize`, `Display`, `std::error::Error`.
-   **`HandlerError`**: Wrapper type for errors returned *by* the handler function (`Err(handler_error)`). Allows retrieving the original error type.
    -   `new<T: Any + Send + Sync + 'static>(val: T) -> HandlerError`
    -   `get<T: Any + Send + Sync + 'static>(&self) -> Option<&T>`: Attempt to downcast the contained error back to its original type `T`.
    -   `remove<T: Any + Send + Sync + 'static>(&mut self) -> Option<T>`: Downcast and take ownership.
    -   `type_name(&self) -> &'static str`: Get the type name of the contained error.
    -   Implements `Debug`, `Serialize` (serializes type name info), `Display`, `std::error::Error`.
-   **`IntoHandlerError` Trait**: Converts any error (`T: Any + Send + Sync + 'static`) into a `HandlerError`. Automatically implemented for types meeting the bounds (including `HandlerError`, `String`, `&'static str`, `Value`). Allows handlers to use `?` or return `Result<MySuccess, MyCustomError>` which automatically converts `MyCustomError` into `HandlerError` when the handler signature specifies `-> HandlerResult<MySuccess>`.
-   **`#[derive(RpcHandlerError)]`** (requires `rpc-router-macros` feature): Derive macro to implement `std::error::Error` and `IntoHandlerError` (via `From<YourEnumVariant> for HandlerError`) for custom error enums returned by handlers. This simplifies returning custom errors.

## JSON-RPC Response (`RpcResponse`, `RpcError`)

These types represent the final JSON-RPC 2.0 response sent back to the client. They are typically created by converting from `CallResult`.

-   **`RpcResponse`**: Enum representing a standard JSON-RPC 2.0 response. Contains either `RpcSuccessResponse` or `RpcErrorResponse`.
    ```rust
    pub enum RpcResponse {
        Success(RpcSuccessResponse),
        Error(RpcErrorResponse),
    }
    ```
    -   Implements `Serialize`, `Deserialize`, `Debug`, `Clone`, `PartialEq`.
    -   Provides `from_success(id, result)` and `from_error(id, error)` constructors.
    -   Provides `id()`, `is_success()`, `is_error()`, `into_parts()`.
-   **`RpcSuccessResponse`**: Struct holding the `id` and `result` (a `Value`) for a successful response.
    ```rust
    pub struct RpcSuccessResponse {
        pub id: RpcId,
        pub result: Value,
    }
    ```
-   **`RpcErrorResponse`**: Struct holding the `id` and `error` (`RpcError`) for an error response.
    ```rust
    pub struct RpcErrorResponse {
        pub id: RpcId,
        pub error: RpcError,
    }
    ```
-   **`RpcError`**: Struct representing the JSON-RPC 2.0 Error Object.
    ```rust
    pub struct RpcError {
        pub code: i64,
        pub message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub data: Option<Value>,
    }
    ```
    -   Includes constants for standard JSON-RPC error codes (`CODE_PARSE_ERROR`, `CODE_INVALID_REQUEST`, `CODE_METHOD_NOT_FOUND`, `CODE_INVALID_PARAMS`, `CODE_INTERNAL_ERROR`).
    -   Includes constructor functions like `from_parse_error(data)`, `from_invalid_request(data)`, etc.
    -   Implements `Serialize`, `Deserialize`, `Debug`, `Clone`, `PartialEq`.
-   **Conversion**:
    -   `From<CallSuccess> for RpcResponse`: Creates `RpcResponse::Success`.
    -   `From<CallError> for RpcResponse`: Creates `RpcResponse::Error` by converting `CallError.error` into an `RpcError`.
    -   `From<CallResult> for RpcResponse`: Handles `Ok(CallSuccess)` and `Err(CallError)`.
    -   `From<&router::Error> for RpcError`: Converts internal `router::Error` variants into standard `RpcError` structures (mapping codes and messages, placing original error string representation in `data`).
    -   `From<CallError> for RpcError`: Converts `CallError` into `RpcError`.
    -   `From<&CallError> for RpcError`.
-   **Serialization/Deserialization**: `RpcResponse` implements `Serialize` and `Deserialize` according to the JSON-RPC 2.0 specification, including strict validation during deserialization (e.g., checks for `"jsonrpc": "2.0"`, presence of `id`, mutual exclusion of `result` and `error`).
-   **`RpcResponseParsingError`**: Enum detailing errors during `RpcResponse` deserialization (e.g., `InvalidJsonRpcVersion`, `BothResultAndError`, `MissingId`, `InvalidErrorObject`).


## Usage Example

```rust
use rpc_router::{resources_builder, router_builder, FromResources, IntoParams, HandlerResult, RpcRequest, RpcResource, RpcParams, RpcResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;

// --- Define Resources
#[derive(Clone, RpcResource)]
struct AppState {
    version: String,
}

#[derive(Clone, RpcResource)]
struct DbPool {
    url: String, 
}

// --- Define Params
#[derive(Deserialize, Serialize, RpcParams)]
struct HelloParams {
    name: String,
}

// --- Define Custom Error
#[derive(Debug, thiserror::Error, rpc_router::RpcHandlerError)]
enum MyHandlerError {
    #[error("Something went wrong: {0}")]
    SpecificError(String),
}

// --- Define RPC Handlers
async fn hello(state: AppState, params: HelloParams) -> HandlerResult<String> {
    Ok(format!("Hello {}, from app version {}!", params.name, state.version))
}

async fn get_db_url(db: DbPool) -> HandlerResult<String> {
    Ok(db.url.clone())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Create Resources
    let resources = resources_builder![
        AppState { version: "1.0".to_string() },
        DbPool { url: "dummy-db-url".to_string() }
    ].build();

    // --- Create Router
    let router = router_builder![
        handlers: [hello, get_db_url]
    ].build();

    // --- Simulate an RPC Call
    let request_json = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "hello",
        "params": {"name": "World"}
    });
    let rpc_request = RpcRequest::from_value(request_json)?;

    // Execute the call
    let call_result = router.call_with_resources(rpc_request, resources).await;

    // --- Process the Result
    let rpc_response = RpcResponse::from(call_result);
    println!("Response: {}", serde_json::to_string_pretty(&rpc_response)?);

    Ok(())
}
```

## Utility Macros

- **`router_builder!(...)`**: Creates a `RouterBuilder` (see Router Configuration).
- **`resources_builder!(...)`**: Creates a `ResourcesBuilder` (see Resources Management).
- **`#[derive(RpcParams)]`**: Implements `IntoParams`.
- **`#[derive(RpcResource)]`**: Implements `FromResources`.
- **`#[derive(RpcHandlerError)]`**: Implements error boilerplate for handler errors (`IntoHandlerError`).
