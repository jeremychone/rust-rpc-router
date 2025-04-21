# `rpc-router` - All APIs

This document provides a detailed overview of the public APIs exposed by the `rpc-router` crate.

## Table of Contents

- [Core Concepts](#core-concepts)
  - [`Router`](#router)
  - [`RouterBuilder`](#routerbuilder)
  - [`Handler` Trait](#handler-trait)
  - [`Resources`](#resources)
  - [`ResourcesBuilder`](#resourcesbuilder)
  - [`FromResources` Trait](#fromresources-trait)
  - [`RpcResource` Derive Macro](#rpcresource-derive-macro)
  - [`IntoParams` Trait](#intoparams-trait)
  - [`IntoDefaultRpcParams` Trait](#intodefaultrpcparams-trait)
  - [`RpcParams` Derive Macro](#rpcparams-derive-macro)
- [Request Handling](#request-handling)
  - [`RpcRequest`](#rpcrequest)
  - [`RpcId`](#rpcid)
  - [`RpcRequestParsingError`](#rpcrequestparsingerror)
- [Response Handling](#response-handling)
  - [`CallResult`](#callresult)
  - [`CallSuccess`](#CallSuccess)
  - [`CallError`](#callerror)
  - [`RpcResponse`](#rpcresponse)
  - [`RpcError`](#rpcerror)
  - [`RpcResponseParsingError`](#rpcresponseparsingerror)
- [Error Handling](#error-handling)
  - [`rpc_router::Error`](#rpc_routererror)
  - [`HandlerError`](#handlererror)
  - [`HandlerResult`](#handlerresult)
  - [`IntoHandlerError` Trait](#intohandlererror-trait)
  - [`RpcHandlerError` Derive Macro](#rpchandlererror-derive-macro)
- [Helper Macros](#helper-macros)
  - [`router_builder!`](#router_builder)
  - [`resources_builder!`](#resources_builder)

---

## Core Concepts

### `Router`

The main entry point for handling RPC calls. It stores the routing table and base resources.

```rust
pub struct Router { /* fields omitted */ }

impl Router {
    /// Creates a new `RouterBuilder`.
    pub fn builder() -> RouterBuilder;

    /// Executes an RPC call based on an `RpcRequest`.
    /// Uses the router's base resources.
    pub async fn call(&self, rpc_request: RpcRequest) -> CallResult;

    /// Executes an RPC call, overlaying `additional_resources`
    /// on top of the router's base resources.
    pub async fn call_with_resources(
        &self,
        rpc_request: RpcRequest,
        additional_resources: Resources
    ) -> CallResult;

    /// Lower-level call execution taking individual components.
    /// Uses the router's base resources. `id` defaults to `Null` if `None`.
    pub async fn call_route(
        &self,
        id: Option<RpcId>,
        method: impl Into<String>,
        params: Option<Value>
    ) -> CallResult;

    /// Lower-level call execution with overlay resources.
    /// `id` defaults to `Null` if `None`.
    pub async fn call_route_with_resources(
        &self,
        id: Option<RpcId>,
        method: impl Into<String>,
        params: Option<Value>,
        additional_resources: Resources
    ) -> CallResult;
}

// Router itself can be a resource
impl FromResources for Router {}
```

- **Cloning:** `Router` uses `Arc` internally, making clones cheap and suitable for sharing across threads/tasks.
- **Call Methods:** Provide high-level (`call`, `call_with_resources`) and lower-level (`call_route`, `call_route_with_resources`) ways to execute RPC calls.
- **Resources:** Can hold base resources accessible to all handlers. `call_with_resources` allows providing request-specific resources.

### `RouterBuilder`

Used to configure and build a `Router`.

```rust
pub struct RouterBuilder { /* fields omitted */ }

impl RouterBuilder {
    /// Adds a handler function directly.
    /// Generic, may lead to monomorphization for each handler type.
    pub fn append<F, T, P, R>(
        mut self,
        name: &'static str,
        handler: F
    ) -> Self
    where F: Handler<T, P, R> + Clone + Send + Sync + 'static, T: ..., P: ..., R: ...;

    /// Adds a pre-boxed handler trait object. Preferred to avoid monomorphization.
    pub fn append_dyn(
        mut self,
        name: &'static str,
        dyn_handler: Box<dyn RpcHandlerWrapperTrait>
    ) -> Self;

    /// Adds a resource to the router's base resources.
    pub fn append_resource<T>(mut self, val: T) -> Self
    where T: FromResources + Clone + Send + Sync + 'static;

    /// Merges another `RouterBuilder`'s routes and resources into this one.
    pub fn extend(mut self, other_builder: RouterBuilder) -> Self;

    /// Extends the base resources with those from an optional `ResourcesBuilder`.
    pub fn extend_resources(
        mut self,
        resources_builder: Option<ResourcesBuilder>
    ) -> Self;

    /// Replaces the router's base resources with those from a `ResourcesBuilder`.
    pub fn set_resources(mut self, resources_builder: ResourcesBuilder) -> Self;

    /// Consumes the builder and creates the `Router`.
    pub fn build(self) -> Router;
}
```

- **Fluent Interface:** Methods return `Self` for chaining.
- **Handlers:** Add handlers using `append` (generic) or `append_dyn` (trait object).
- **Resources:** Manage base resources with `append_resource`, `extend_resources`, `set_resources`.
- **Composition:** Use `extend` to combine multiple router configurations.

### `Handler` Trait

The core trait implemented by RPC handler functions. You typically don't implement this directly; the library provides implementations for `async fn` with specific signatures.

```rust
pub trait Handler<T, P, R>: Clone
where T: Send + Sync + 'static, P: Send + Sync + 'static, R: Send + Sync + 'static {
    type Future: Future<Output = Result<Value>> + Send + 'static;

    fn call(self, rpc_resources: Resources, params: Option<Value>) -> Self::Future;

    fn into_dyn(self) -> Box<dyn RpcHandlerWrapperTrait>
    where Self: Sized + Send + Sync + 'static;
}
```

- **Signature:** The library implements `Handler` for functions like:
    - `async fn()` -> `HandlerResult<Serialize>`
    - `async fn(R1)` -> `HandlerResult<Serialize>`
    - `async fn(R1, R2)` -> `HandlerResult<Serialize>`
    - ... (up to 8 resource parameters `R` where `Ri: FromResources`)
    - `async fn(P)` -> `HandlerResult<Serialize>`
    - `async fn(R1, P)` -> `HandlerResult<Serialize>`
    - `async fn(R1, R2, P)` -> `HandlerResult<Serialize>`
    - ... (up to 8 resource parameters `R`, followed by one param `P: IntoParams`)
- **`T`:** Tuple representing the `FromResources` types.
- **`P`:** The `IntoParams` type (or `()` if no params).
- **`R`:** The `Serialize` type returned within `Ok`.
- **Return Type:** Must be `HandlerResult<S>` where `S: Serialize`. `HandlerResult<T>` is `Result<T, HandlerError>`.
- **`into_dyn`:** Used internally to convert the handler function into a type-erased trait object for storage in the router.

### `Resources`

A type map holding shared application state or request-specific data.

```rust
#[derive(Debug, Clone, Default)]
pub struct Resources { /* fields omitted */ }

impl Resources {
    /// Creates a new `ResourcesBuilder`.
    pub fn builder() -> ResourcesBuilder;

    /// Retrieves a clone of a resource of type `T`.
    /// Checks overlay resources first, then base resources.
    pub fn get<T: Clone + Send + Sync + 'static>(&self) -> Option<T>;

    /// Checks if both base and overlay resources are empty.
    pub fn is_empty(&self) -> bool;
}
```

- **Type Map:** Stores instances of different types, retrievable by their type ID.
- **Cloning:** Uses `Arc` internally for cheap cloning. Resources themselves must be `Clone`.
- **Layered:** Can have base resources (from the `Router`) and overlay resources (provided per-call). `get` prioritizes the overlay.

### `ResourcesBuilder`

Used to construct a `Resources` instance.

```rust
#[derive(Debug, Default, Clone)]
pub struct ResourcesBuilder { /* fields omitted */ }

impl ResourcesBuilder {
    /// Retrieves a clone of a resource added to the builder.
    pub fn get<T: Clone + Send + Sync + 'static>(&self) -> Option<T>;

    /// Adds a resource instance to the builder (consuming).
    pub fn append<T: Clone + Send + Sync + 'static>(mut self, val: T) -> Self;

    /// Adds a resource instance to the builder (mutable borrow).
    pub fn append_mut<T: Clone + Send + Sync + 'static>(&mut self, val: T);

    /// Consumes the builder and creates a `Resources` instance.
    pub fn build(self) -> Resources;
}
```

- **Fluent Interface:** `append` returns `Self`.
- **Builds `Resources`:** Collects resources and then uses `build()` to create the final `Resources` object.

### `FromResources` Trait

Marks a type as extractable from `Resources`.

```rust
pub trait FromResources {
    /// Attempts to retrieve the resource from the `Resources` map.
    /// Default implementation provided.
    fn from_resources(resources: &Resources) -> FromResourcesResult<Self>
    where Self: Sized + Clone + Send + Sync + 'static;
}

// Blanket implementation for Option<T> where T: FromResources
impl<T> FromResources for Option<T> where T: FromResources, ... {}
```

- **Implementation:** You usually derive this using `#[derive(RpcResource)]` or implement it manually if needed. The trait itself doesn't require methods if using the default.
- **Handler Parameters:** Types implementing `FromResources` can appear as parameters (before the optional `IntoParams` parameter) in handler functions.
- **`Option<T>`:** If a handler takes `Option<T>` (where `T: FromResources`), it will receive `Some(T)` if the resource exists, and `None` otherwise, without causing an error.

### `RpcResource` Derive Macro

A convenience macro to derive `FromResources` for a struct or enum.

```rust
use rpc_router::RpcResource;

#[derive(Clone, RpcResource)]
struct MyDbConnection { /* ... */ }
```

- **Requires:** The type must also be `Clone + Send + Sync + 'static`.

### `IntoParams` Trait

Marks a type as deserializable from the JSON-RPC `params` field.

```rust
pub trait IntoParams: DeserializeOwned + Send {
    /// Converts the `Option<Value>` from the request into `Self`.
    /// Default implementation attempts deserialization, errors if `value` is `None`.
    fn into_params(value: Option<Value>) -> Result<Self>;
}
```

- **Implementation:** Typically derived using `#[derive(RpcParams)]` for simple `serde` deserialization. Implement manually for custom parsing logic or validation.
- **Handler Parameter:** A type implementing `IntoParams` can be the *last* parameter in a handler function.
- **Default Behavior:** If `params` are missing in the request but the handler expects them, the default implementation returns `Error::ParamsMissingButRequested`.

### `IntoDefaultRpcParams` Trait

A marker trait used with `IntoParams` to provide default values.

```rust
pub trait IntoDefaultRpcParams: DeserializeOwned + Send + Default {}

// Blanket implementation of IntoParams for types implementing IntoDefaultRpcParams
impl<P> IntoParams for P where P: IntoDefaultRpcParams {
    fn into_params(value: Option<Value>) -> Result<Self> {
        match value {
            Some(value) => Ok(serde_json::from_value(value).map_err(Error::ParamsParsing)?),
            None => Ok(Self::default()), // Use default() if params are missing
        }
    }
}
```

- **Usage:** Implement this marker trait on your `IntoParams` type *if* it also implements `Default`.
- **Behavior:** If the JSON-RPC request omits the `params` field, the handler will receive `T::default()` instead of an error.

### `RpcParams` Derive Macro

A convenience macro to derive `IntoParams` (and potentially `IntoDefaultRpcParams` if `Default` is also derived/implemented).

```rust
use rpc_router::RpcParams;
use serde::Deserialize;

#[derive(Deserialize, RpcParams)] // Implements IntoParams
struct MyParams {
    field: String,
}

#[derive(Deserialize, Default, RpcParams)] // Implements IntoParams + IntoDefaultRpcParams
struct MyOptionalParams {
    field: Option<i32>,
}
```

- **Requires:** The type must implement `serde::Deserialize` and be `Send + 'static`.
- **Default Handling:** If the type also implements `Default`, the macro ensures the `IntoDefaultRpcParams` behavior is used.

---

## Request Handling

Types related to parsing and representing incoming JSON-RPC requests.

### `RpcRequest`

Represents a parsed JSON-RPC request (excluding notifications).

```rust
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RpcRequest {
    pub id: RpcId,
    pub method: String,
    pub params: Option<Value>,
}

impl RpcRequest {
    /// Parses a `serde_json::Value` into an `RpcRequest`.
    /// Validates `jsonrpc: "2.0"` and presence/type of `id` and `method`.
    pub fn from_value(value: Value) -> Result<RpcRequest, RpcRequestParsingError>;
}

impl TryFrom<Value> for RpcRequest {
    type Error = RpcRequestParsingError;
    fn try_from(value: Value) -> Result<RpcRequest, RpcRequestParsingError>;
}
```

- **Structure:** Holds the core components of a standard JSON-RPC request.
- **Parsing:** Use `RpcRequest::from_value` or `value.try_into()` for strict parsing and validation. Standard `serde_json::from_value` can also be used but won't validate `jsonrpc` version.

### `RpcId`

Represents a JSON-RPC request ID (String, Number, or Null).

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RpcId {
    String(Arc<str>), // Uses Arc<str> for efficient cloning
    Number(i64),
    Null,
}

// Implements: Serialize, Deserialize, Display, Default, From<String>, From<&str>, From<i64>, ...
impl RpcId {
    pub fn to_value(&self) -> Value;
    pub fn from_value(value: Value) -> Result<Self, RpcRequestParsingError>;
}
```

- **Types:** Enforces the valid JSON-RPC ID types.
- **Efficiency:** Uses `Arc<str>` for string IDs to avoid reallocations on cloning.

### `RpcRequestParsingError`

Errors that occur during `RpcRequest::from_value` parsing.

```rust
#[derive(Debug, Serialize)]
pub enum RpcRequestParsingError {
    RequestInvalidType { actual_type: String },
    VersionMissing { id: Option<Value>, method: Option<String> },
    VersionInvalid { id: Option<Value>, method: Option<String>, version: Value },
    MethodMissing { id: Option<Value> },
    MethodInvalidType { id: Option<Value>, method: Value },
    MethodInvalid { actual: String }, // Added variant
    IdMissing { method: Option<String> },
    IdInvalid { actual: String, cause: String },
    Parse(#[serde_as(as = "DisplayFromStr")] serde_json::Error),
}
// Implements: Display, std::error::Error
```

- **Contextual:** Provides details about why parsing failed (missing fields, wrong types, invalid version).

---

## Response Handling

Types related to the results of router calls and standard JSON-RPC responses.

### `CallResult`

The type alias for the `Result` returned by `Router::call` methods.

```rust
pub type CallResult = Result<CallSuccess, CallError>;
```

### `CallSuccess`

Represents a successful result from a `Router::call`.

```rust
#[derive(Debug, Clone)]
pub struct CallSuccess {
    pub id: RpcId,
    pub method: String,
    pub value: Value, // The JSON-serialized return value from the handler
}
```

- **Context:** Includes the original `id` and `method` for logging/tracing.
- **Value:** Contains the successful result from the handler, already serialized to `serde_json::Value`.

### `CallError`

Represents a failed result from a `Router::call`.

```rust
#[derive(Debug)]
pub struct CallError {
    pub id: RpcId,
    pub method: String,
    pub error: crate::Error, // The specific rpc-router::Error that occurred
}
// Implements: Display, std::error::Error
```

- **Context:** Includes the original `id` and `method`.
- **Error:** Contains the specific `rpc_router::Error` variant indicating the failure reason (e.g., method unknown, parameter parsing failed, handler error).

### `RpcResponse`

Represents a standard JSON-RPC 2.0 response object (Success or Error).

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum RpcResponse {
    Success { id: RpcId, result: Value },
    Error { id: RpcId, error: RpcError },
}

// Implements: Serialize, Deserialize
impl RpcResponse {
    pub fn from_success(id: RpcId, result: Value) -> Self;
    pub fn from_error(id: RpcId, error: RpcError) -> Self;
    pub fn is_success(&self) -> bool;
    pub fn is_error(&self) -> bool;
    pub fn id(&self) -> &RpcId;
    pub fn into_parts(self) -> (RpcId, Result<Value, RpcError>);
}

// Conversion from router results
impl From<CallSuccess> for RpcResponse;
impl From<CallError> for RpcResponse;
impl From<CallResult> for RpcResponse;
```

- **Standard:** Conforms to the JSON-RPC 2.0 specification for response objects.
- **Serialization:** Serializes to the correct JSON-RPC format (including `jsonrpc: "2.0"`).
- **Deserialization:** Parses JSON into `RpcResponse`, validating the structure.
- **Conversion:** Easily created from `CallResult`, `CallSuccess`, or `CallError`.

### `RpcError`

Represents the standard JSON-RPC 2.0 Error Object structure.

```rust
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl RpcError {
    // Constants for standard codes (-32700, -32600, -32601, -32602, -32603)
    pub const CODE_PARSE_ERROR: i64 = -32700;
    // ... other codes

    // Constructors for standard errors
    pub fn from_parse_error(data: Option<Value>) -> Self;
    // ... other constructors
}

// Conversion from router errors
impl From<&crate::Error> for RpcError;
impl From<CallError> for RpcError;
impl From<&CallError> for RpcError;
```

- **Standard Fields:** `code`, `message`, optional `data`.
- **Predefined Codes:** Provides constants and constructors for standard JSON-RPC error codes.
- **Conversion:** Can be created from `rpc_router::Error` or `CallError`, mapping router-internal errors to appropriate JSON-RPC error codes and messages. The original router error is often included as a string in the `data` field.

### `RpcResponseParsingError`

Errors that occur during `RpcResponse` deserialization.

```rust
#[derive(Debug, Serialize)]
pub enum RpcResponseParsingError {
    InvalidJsonRpcVersion { id: Option<RpcId>, expected: &'static str, actual: Option<Value> },
    MissingJsonRpcVersion { id: Option<RpcId> },
    MissingId,
    InvalidId(#[serde_as(as = "DisplayFromStr")] crate::RpcRequestParsingError), // Reuses ID parsing error
    MissingResultAndError { id: RpcId },
    BothResultAndError { id: RpcId },
    InvalidErrorObject(#[serde_as(as = "DisplayFromStr")] serde_json::Error),
    Serde(#[serde_as(as = "DisplayFromStr")] serde_json::Error),
}
// Implements: Display, std::error::Error, From<serde_json::Error>, From<RpcRequestParsingError>
```

- **Specific:** Details why response parsing failed (e.g., missing `id`, both `result` and `error` present, invalid version).

---

## Error Handling

Types related to errors within the router and handlers.

### `rpc_router::Error`

The primary error enum for the router itself. Returned within `CallError`.

```rust
#[derive(Debug, Serialize)]
pub enum Error {
    // -- Parameter Errors
    ParamsParsing(#[serde_as(as = "DisplayFromStr")] serde_json::Error),
    ParamsMissingButRequested,

    // -- Router Errors
    MethodUnknown,

    // -- Handler Errors
    FromResources(FromResourcesError), // Error getting resource
    HandlerResultSerialize(#[serde_as(as = "DisplayFromStr")] serde_json::Error), // Error serializing handler Ok result
    Handler(HandlerError), // Wrapper for error returned by the handler itself
}
// Implements: Display, std::error::Error, From<HandlerError>, From<FromResourcesError>
```

- **Categorized:** Groups errors related to parameter handling, routing, resource extraction, and handler execution.
- **`Handler(HandlerError)`:** This variant wraps errors returned directly from your handler function, allowing the application to potentially recover the original error type.

### `HandlerError`

A type-erased wrapper for errors returned by handler functions.

```rust
#[derive(Debug)]
pub struct HandlerError { /* fields omitted */ }

// Implements: Serialize, Display, std::error::Error
impl HandlerError {
    /// Creates a new `HandlerError` wrapping the given value.
    pub fn new<T>(val: T) -> HandlerError where T: Any + Send + Sync;

    /// Attempts to downcast the wrapped error to type `T`.
    pub fn get<T: Any + Send + Sync>(&self) -> Option<&T>;

    /// Attempts to downcast and take ownership of the wrapped error.
    pub fn remove<T: Any + Send + Sync>(&mut self) -> Option<T>;

    /// Returns the type name of the wrapped error.
    pub fn type_name(&self) -> &'static str;
}
```

- **Type Erasure:** Allows the router to handle different error types from handlers uniformly.
- **Recovery:** Use `get::<MyError>()` or `remove::<MyError>()` to attempt retrieving the original error type you returned from the handler.
- **Serialization:** By default, serializes to a string indicating the contained type name (used when converting to `RpcError`).

### `HandlerResult<T>`

The required return type alias for RPC handler functions.

```rust
pub type HandlerResult<T> = Result<T, HandlerError>;
```

- **Usage:** Your `async fn` handlers must return `HandlerResult<S>` where `S: Serialize`.

### `IntoHandlerError` Trait

Trait used to convert various error types into `HandlerError`.

```rust
pub trait IntoHandlerError where Self: Sized + Send + Sync + 'static {
    /// Default implementation wraps `self` in `HandlerError::new(self)`.
    fn into_handler_error(self) -> HandlerError;
}

// Blanket implementations provided for:
// - HandlerError (identity conversion)
// - String
// - &'static str
// - Value
// - Any type T that is Sized + Send + Sync + 'static (via the default method)
```

- **Convenience:** Allows returning types like `String` directly from a handler using `?` (as long as the handler signature is `HandlerResult<T>`). The `?` operator triggers the conversion via `From<E> for HandlerError`, which uses this trait.
- **Custom Errors:** Your custom error types automatically implement this trait via the blanket implementation, enabling them to be returned directly from handlers.

### `RpcHandlerError` Derive Macro

A convenience macro to automatically implement `IntoHandlerError` and potentially `std::error::Error` and `Display` for your custom error enum.

```rust
use rpc_router::RpcHandlerError;
use thiserror::Error; // Example using thiserror

#[derive(Debug, Error, RpcHandlerError)] // Derives IntoHandlerError
enum MyCustomError {
    #[error("Something failed: {0}")]
    Failed(String),
    #[error("IO Error")]
    Io(#[from] std::io::Error), // Example using thiserror's from
}
```

- **Simplifies Boilerplate:** Ensures your error can be seamlessly returned from handlers.

---

## Helper Macros

Macros for reducing boilerplate during setup.

### `router_builder!`

Creates a `RouterBuilder` and adds handlers/resources concisely.

```rust
// Pattern 1: Just handlers (function names become method names)
let router = router_builder!(handler_one, handler_two);

// Pattern 2: Handlers and resources
let router = router_builder!(
    handlers: [handler_one, handler_two],
    resources: [MyResource::new(), AnotherResource::default()]
);

// Pattern 3: Just handlers (alternative syntax)
let router = router_builder!(handlers: [handler_one, handler_two]);
```

- **Syntax:** Supports multiple patterns for defining handlers and associated resources.

### `resources_builder!`

Creates a `ResourcesBuilder` and adds resources concisely.

```rust
let resources = resources_builder!(
    MyResource::new(),
    AnotherResource::default()
).build();
```

- **Simplified Resource Setup:** Useful for creating `Resources` instances independently.
