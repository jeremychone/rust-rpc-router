# RPC Router - All APIs

This document provides a comprehensive overview of all public APIs exposed by the `rpc-router` crate.

## Core Types

-   **`Router`**: The central component that holds RPC method routes and associated handlers. It's typically created using `RouterBuilder` and wrapped in an `Arc` for efficient sharing.
-   **`RouterBuilder`**: Used to configure and build a `Router`. Allows adding handlers and base resources.
-   **`Resources`**: A type map used to hold shared application state or resources (like database connections, configuration, etc.) accessible by handlers.
-   **`ResourcesBuilder`**: Used to configure and build `Resources`.

## Request Handling Flow

1.  **Parsing the Request**: Incoming JSON-RPC requests (as `serde_json::Value`) are typically parsed into an `RpcRequest` object.
2.  **Calling the Router**: The `Router::call` or `Router::call_with_resources` method is invoked with the `RpcRequest` and optional additional `Resources`.
3.  **Method Routing**: The router finds the handler registered for the requested `method`.
4.  **Resource Injection**: The router attempts to extract required resources (types implementing `FromResources`) from the provided `Resources`.
5.  **Parameter Deserialization**: If the handler expects parameters, the `params` field of the `RpcRequest` (an `Option<Value>`) is deserialized into the expected type using the `IntoParams` trait.
6.  **Handler Execution**: The asynchronous handler function is called with the injected resources and deserialized parameters.
7.  **Result Handling**: The handler returns a `HandlerResult<T>` (which is `Result<T, HandlerError>`).
8.  **Router Response**: The router captures the handler's result or any errors during resource/parameter handling and wraps it in a `CallResult` (`Result<CallSuccess, CallError>`).
9.  **JSON-RPC Response Formation**: The `CallResult` is typically converted into a standard JSON-RPC `RpcResponse` (Success or Error) for sending back to the client.

## Request Parsing (`RpcRequest`)

-   **`RpcRequest`**: Represents a parsed JSON-RPC request.
    ```rust
    pub struct RpcRequest {
        pub id: RpcId,
        pub method: String,
        pub params: Option<Value>,
    }
    ```
-   **`RpcId`**: An enum representing the JSON-RPC ID (`String(Arc<str>)`, `Number(i64)`, or `Null`). Implements `Serialize`, `Deserialize`, `From<String>`, `From<&str>`, `From<i64>`, etc.
-   **Parsing**:
    -   **`RpcRequest::from_value(value: Value) -> Result<RpcRequest, RpcRequestParsingError>`**: Parses a `serde_json::Value` into an `RpcRequest`. Performs strict validation: checks for `"jsonrpc": "2.0"` and ensures the `id` is a valid JSON-RPC ID (string, number, or null).
    -   **`RpcRequest::from_value_with_checks(value: Value, checks: RpcRequestCheckFlags) -> Result<RpcRequest, RpcRequestParsingError>`**: Allows selective validation based on flags.
        -   `RpcRequestCheckFlags`: Bitflags to control checks (`VERSION`, `ID`, `ALL`).
        -   Example (skip ID check):
            ```rust
            use rpc_router::{RpcRequest, RpcRequestCheckFlags, RpcRequestParsingError};
            use serde_json::json;

            let request_value = json!({
              "jsonrpc": "2.0",
              // "id": 123, // ID missing, but we'll skip the check
              "method": "my_method",
              "params": [1, 2]
            });

            let flags = RpcRequestCheckFlags::VERSION; // Only check version
            let result = RpcRequest::from_value_with_checks(request_value, flags);

            assert!(result.is_ok());
            let request = result.unwrap();
            // request.id will default to RpcId::Null if check is skipped and id is missing
            assert_eq!(request.id, rpc_router::RpcId::Null);
            assert_eq!(request.method, "my_method");
            ```
    -   **`TryFrom<Value> for RpcRequest`**: Convenience trait implementation calling `RpcRequest::from_value`.
    -   **`RpcRequestParsingError`**: Enum detailing specific parsing failures (e.g., `VersionMissing`, `IdInvalid`, `MethodMissing`).

## Router Invocation

-   **`Router::call(&self, rpc_request: RpcRequest) -> impl Future<Output = CallResult>`**: Executes the request using the router's base resources.
-   **`Router::call_with_resources(&self, rpc_request: RpcRequest, additional_resources: Resources) -> impl Future<Output = CallResult>`**: Executes the request, overlaying `additional_resources` on top of the base resources. Resources are looked up first in `additional_resources`, then in the base resources.
-   **`Router::call_route(&self, id: Option<RpcId>, method: impl Into<String>, params: Option<Value>) -> impl Future<Output = CallResult>`**: Lower-level call using individual components instead of `RpcRequest`. Uses base resources. `id` defaults to `RpcId::Null` if `None`.
-   **`Router::call_route_with_resources(...)`**: Like `call_route` but with `additional_resources`.

## Router Call Output

-   **`CallResult`**: Type alias for `Result<CallSuccess, CallError>`.
-   **`CallSuccess`**: Struct containing the successful result details.
    ```rust
    pub struct CallSuccess {
        pub id: RpcId,
        pub method: String,
        pub value: Value, // Serialized result from the handler
    }
    ```
-   **`CallError`**: Struct containing error details.
    ```rust
    pub struct CallError {
        pub id: RpcId,
        pub method: String,
        pub error: router::Error, // The router/handler error
    }
    ```
    Implements `std::error::Error`.

## Defining Handlers

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

### Handler Parameters (`IntoParams`)

-   **`IntoParams` Trait**: Implement this for types you want to use as the `params` argument in your handlers.
    -   Requires `DeserializeOwned + Send`.
    -   Default `into_params` method deserializes from `Some(Value)`, returns `Error::ParamsMissingButRequested` if `None`.
-   **`IntoDefaultRpcParams` Trait**: Marker trait. If a type implements `IntoDefaultRpcParams` and `Default`, `IntoParams` is automatically implemented such that `T::default()` is used when JSON-RPC params are `null` or absent.
-   **Derive Macro `#[derive(RpcParams)]`**: The recommended way to implement `IntoParams` for simple structs. Equivalent to `impl IntoParams for MyType {}`.
-   **Derive Macro `#[derive(RpcParams, Default)]`**: The recommended way to implement `IntoDefaultRpcParams`. Equivalent to `impl IntoDefaultRpcParams for MyType {}` (assuming `Default` is also derived or implemented).
-   **Blanket Impls (Optional Features)**:
    -   `Option<T>`: `IntoParams` is implemented for `Option<T>` where `T: IntoParams`.
    -   `Value`: `IntoParams` is implemented directly for `serde_json::Value`.

### Handler Resources (`FromResources`)

-   **`FromResources` Trait**: Implement this for types you want to inject as resource arguments into your handlers.
    -   Default `from_resources` method retrieves the type from `Resources`. Returns `FromResourcesError::ResourceNotFound` if not found.
-   **Derive Macro `#[derive(RpcResource)]`**: Recommended way. Equivalent to `impl FromResources for MyType {}`. Ensure the type also implements `Clone + Send + Sync + 'static`.
-   **Blanket Impl `Option<T>`**: `FromResources` is implemented for `Option<T>` where `T: FromResources`, allowing optional resource injection (returns `Ok(None)` if the resource `T` is not found).

## Resources Management

-   **`Resources`**: A cloneable type map (`Arc`-based internally).
    -   `Resources::builder() -> ResourcesBuilder`
    -   `get<T: Clone + Send + Sync + 'static>(&self) -> Option<T>`: Retrieves a clone of the resource.
-   **`ResourcesBuilder`**:
    -   `default()`
    -   `append<T: Clone + Send + Sync + 'static>(self, val: T) -> Self`: Adds a resource.
    -   `append_mut<T>(&mut self, val: T)`: Adds a resource without consuming the builder.
    -   `build(self) -> Resources`: Creates the `Resources` object.
-   **`resources_builder!` Macro**: Convenience for creating a `ResourcesBuilder` and appending items.
    ```rust
    let resources = resources_builder!(MyDb::new(), MyConfig::load()?).build();
    ```

## Router Configuration (`RouterBuilder`)

-   **`RouterBuilder::default()`**: Creates a new builder.
-   **`append<F, T, P, R>(self, name: &'static str, handler: F) -> Self`**: Adds a handler function directly.
-   **`append_dyn(self, name: &'static str, dyn_handler: Box<dyn RpcHandlerWrapperTrait>) -> Self`**: Adds a type-erased handler (often used with `handler.into_dyn()`). Preferred to avoid monomorphization if adding many routes dynamically.
-   **`append_resource<T>(self, val: T) -> Self`**: Adds a base resource available to all handlers in this router.
-   **`extend(self, other_builder: RouterBuilder) -> Self`**: Merges another builder's routes and resources.
-   **`extend_resources(self, resources_builder: Option<ResourcesBuilder>) -> Self`**: Adds resources from another `ResourcesBuilder` to the base resources.
-   **`set_resources(self, resources_builder: ResourcesBuilder) -> Self`**: Replaces the router's base resources.
-   **`build(self) -> Router`**: Builds the final `Router`.
-   **`router_builder!` Macro**: Convenience macro.
    ```rust
    // Simple list of handlers
    let router = router_builder!(handler_one, handler_two).build();

    // With handlers and resources
    let router = router_builder!(
        handlers: [handler_one, handler_two],
        resources: [MyDb::new()]
    ).build();
    ```

## Error Handling

-   **`router::Error`**: Enum representing errors occurring *within* the router or during handler invocation setup (parameter parsing, resource fetching).
    -   `ParamsParsing(serde_json::Error)`
    -   `ParamsMissingButRequested`
    -   `MethodUnknown`
    -   `FromResources(FromResourcesError)`
    -   `HandlerResultSerialize(serde_json::Error)`: Error serializing the handler's successful `Ok(value)`.
    -   `Handler(HandlerError)`: Wraps an error returned *by* the handler (`Err(handler_error)`).
-   **`FromResourcesError`**: Error specifically from failing to get a resource.
    -   `ResourceNotFound(&'static str)`: Contains the name of the type not found.
-   **`HandlerError`**: Wrapper for errors returned *by* the handler (`Err(handler_error)`).
    -   `new<T: Any + Send + Sync + 'static>(val: T) -> HandlerError`
    -   `get<T: Any + Send + Sync>(&self) -> Option<&T>`: Attempt to downcast the contained error back to its original type `T`.
    -   `remove<T: Any + Send + Sync>(&mut self) -> Option<T>`: Downcast and take ownership.
    -   `type_name(&self) -> &'static str`: Get the type name of the contained error.
-   **`IntoHandlerError` Trait**: Convert any error (`T: Send + Sync + 'static`) into a `HandlerError`. Automatically implemented for types meeting the bounds. Allows handlers to return `Result<MySuccess, MyError>` which automatically converts `MyError` into `HandlerError`.
-   **`#[derive(RpcHandlerError)]`**: Derive macro to implement `std::error::Error` and `From<YourEnumVariant> for HandlerError` for custom error enums returned by handlers.

## JSON-RPC Response (`RpcResponse`)

-   **`RpcResponse`**: Enum representing a standard JSON-RPC 2.0 response.
    -   `Success { id: RpcId, result: Value }`
    -   `Error { id: RpcId, error: RpcError }`
-   **`RpcError`**: Struct representing the JSON-RPC 2.0 Error Object.
    -   `code: i64`
    -   `message: String`
    -   `data: Option<Value>`
    -   Includes constants for standard JSON-RPC error codes (`CODE_METHOD_NOT_FOUND`, etc.).
-   **Conversion**:
    -   `From<CallSuccess> for RpcResponse`
    -   `From<CallError> for RpcResponse`
    -   `From<CallResult> for RpcResponse` (handles Ok/Err)
    -   `From<&router::Error> for RpcError`: Converts internal router errors into standard `RpcError` structures.
    -   `From<CallError> for RpcError`
-   **Serialization/Deserialization**: `RpcResponse` implements `Serialize` and `Deserialize` according to the JSON-RPC 2.0 specification.
-   **`RpcResponseParsingError`**: Enum detailing errors during `RpcResponse` deserialization (e.g., `InvalidJsonRpcVersion`, `BothResultAndError`).

## Utility Macros

-   **`router_builder!(...)`**: Creates a `RouterBuilder` (see Router Configuration).
-   **`resources_builder!(...)`**: Creates a `ResourcesBuilder` (see Resources Management).
-   **`#[derive(RpcParams)]`**: Implements `IntoParams`.
-   **`#[derive(RpcResource)]`**: Implements `FromResources`.
-   **`#[derive(RpcHandlerError)]`**: Implements error boilerplate for handler errors.

This overview covers the primary public APIs. Refer to the specific module documentation (`src/.../mod.rs` or individual files) for more granular details.
