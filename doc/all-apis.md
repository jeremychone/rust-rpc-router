# rpc-router Public API Reference

This document lists all public APIs exported by the `rpc-router` crate, including types, traits, functions, and macros.

## Crate-level Re-exports

- **Error Handling**
  
  - `rpc_router::Error`  
    Enum of routing and handler errors:
    ```rust
    #[serde_as]
    pub enum Error {
        ParamsParsing(serde_json::Error),
        ParamsMissingButRequested,
        MethodUnknown,
        FromResources(FromResourcesError),
        HandlerResultSerialize(serde_json::Error),
        Handler(HandlerError),
    }
    ```
  
  - `rpc_router::Result<T>`  
    Alias for `core::result::Result<T, rpc_router::Error>`.

- **Handler Traits & Types**
  
  - `rpc_router::Handler<T, P, R>`  
    Trait implemented for async handler functions.  
    ```rust
    pub trait Handler<T, P, R>: Clone
    where
        T: Send + Sync + 'static,
        P: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        type Future: Future<Output = Result<Value>> + Send + 'static;
        fn call(self, resources: Resources, params: Option<Value>) -> Self::Future;
        fn into_dyn(self) -> Box<dyn RpcHandlerWrapperTrait>
        where
            Self: Sized + Send + Sync + 'static;
    }
    ```

  - `rpc_router::HandlerError`  
    Wrapper around application errors with methods:
    - `HandlerError::new<T>(val: T)`
    - `get<T>() -> Option<&T>`
    - `remove<T>() -> Option<T>`
    - `type_name() -> &'static str`
  
  - `rpc_router::HandlerResult<T>`  
    Alias for `core::result::Result<T, HandlerError>`.

  - `rpc_router::IntoHandlerError`  
    Trait to convert custom errors into `HandlerError`. Auto‑implemented for `HandlerError`, `String`, `&'static str`, `serde_json::Value`.

- **Parameters Traits**
  
  - `rpc_router::IntoParams`  
    Trait to convert `Option<Value>` into a concrete type via `serde::DeserializeOwned`.  
    ```rust
    pub trait IntoParams: DeserializeOwned + Send {
        fn into_params(value: Option<Value>) -> Result<Self>;
    }
    ```

  - `rpc_router::IntoDefaultRpcParams`  
    Marker trait; types implementing `Default` can omit params and use `Default::default()`.

  - Blanket implementations of `IntoParams` for:
    
    - `Option<D>`  
    - `serde_json::Value`

- **Request & ID Types**

  - `rpc_router::RpcId`
    Enum representing a JSON-RPC request ID (String, Number, Null).
    ```rust
    pub enum RpcId {
        String(Arc<str>),
        Number(i64),
        Null,
    }
    impl RpcId {
        pub fn from_value(value: Value) -> Result<Self, RequestParsingError>;
        pub fn to_value(&self) -> Value;
    }
    // Also implements Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, Display, Default
    // Also implements From<String>, From<&str>, From<i64>, From<i32>, From<u32>
    ```

  - `rpc_router::Request`  
    Represents a JSON‑RPC 2.0 request:
    ```rust
    pub struct Request {
        pub id: RpcId, // Changed from Value
        pub method: String,
        pub params: Option<Value>,
    }
    impl Request {
        pub fn from_value(value: Value) -> Result<Request, RequestParsingError>;
    }
    ```
  
  - `impl TryFrom<serde_json::Value> for Request`  
    Convenient conversion from a raw `Value`, performing JSON‑RPC 2.0 validation (including ID parsing).
  
  - `rpc_router::RequestParsingError`  
    Enum of request parsing failures:
    ```rust
    pub enum RequestParsingError {
        RequestInvalidType { actual_type: String },
        VersionMissing { id: Option<Value>, method: Option<String> },
        VersionInvalid { id: Option<Value>, method: Option<String>, version: Value },
        MethodMissing { id: Option<Value> },
        MethodInvalidType { id: Option<Value>, method: Value },
        MethodInvalid { actual: String }, // Note: Method name checked by Router, not here
        IdMissing { method: Option<String> },
        IdInvalid { actual: String, cause: String }, // Added RpcId parsing failure
        Parse(serde_json::Error),
    }
    ```

- **Resources Traits & Types**
  
  - `rpc_router::FromResources`  
    Trait to extract a resource from `Resources`; blanket impl for `Option<T>`.
  
  - `rpc_router::FromResourcesError`  
    Enum with variant `ResourceNotFound(&'static str)`.
  
  - `rpc_router::ResourcesBuilder`  
    Builder for collecting resources:
    ```rust
    pub struct ResourcesBuilder { /* … */ }
    impl ResourcesBuilder {
        // Default::default() can be used instead of builder()
        pub fn append<T>(self, val: T) -> Self;
        pub fn append_mut<T>(&mut self, val: T);
        pub fn get<T>(&self) -> Option<T>;
        pub fn build(self) -> Resources;
    }
    ```
  
  - `rpc_router::Resources`  
    Immutable, shareable collection of resources:
    ```rust
    pub struct Resources { /* … */ }
    impl Resources {
        pub fn builder() -> ResourcesBuilder;
        pub fn get<T>(&self) -> Option<T>;
        pub fn is_empty(&self) -> bool;
    }
    ```

- **Router Types & Builders**
  
  - `rpc_router::Router`  
    Main entry point to register handlers and execute calls:
    ```rust
    pub struct Router { /* … */ }
    impl Router {
        pub fn builder() -> RouterBuilder;
        pub async fn call(&self, request: Request) -> CallResult;
        pub async fn call_with_resources(&self, request: Request, additional: Resources) -> CallResult;
        pub async fn call_route(&self, id: Option<RpcId>, method: impl Into<String>, params: Option<Value>) -> CallResult; // Changed id type
        pub async fn call_route_with_resources(&self, id: Option<RpcId>, method: impl Into<String>, params: Option<Value>, additional: Resources) -> CallResult; // Changed id type
    }
    impl FromResources for Router {}
    ```
  
  - `rpc_router::RouterBuilder`  
    Builder for `Router`:
    ```rust
    pub struct RouterBuilder { /* … */ }
    impl RouterBuilder {
        pub fn append_dyn(self, name: &'static str, handler: Box<dyn RpcHandlerWrapperTrait>) -> Self;
        pub fn append<F, T, P, R>(self, name: &'static str, handler: F) -> Self
            where F: Handler<T, P, R> + Clone + Send + Sync + 'static, …;
        pub fn extend(self, other: RouterBuilder) -> Self;
        pub fn append_resource<T>(self, val: T) -> Self where T: Clone + Send + Sync + 'static; // Removed FromResources constraint (added in .build())
        pub fn extend_resources(self, resources: Option<ResourcesBuilder>) -> Self;
        pub fn set_resources(self, resources: ResourcesBuilder) -> Self;
        pub fn build(self) -> Router;
    }
    ```

- **Call Response & Error**
  
  - `rpc_router::CallResponse`  
    ```rust
    pub struct CallResponse {
        pub id: RpcId, // Changed from Value
        pub method: String,
        pub value: Value,
    }
    ```
  
  - `rpc_router::CallError`  
    ```rust
    pub struct CallError {
        pub id: RpcId, // Changed from Value
        pub method: String,
        pub error: rpc_router::Error,
    }
    ```
  
  - `rpc_router::CallResult`  
    Alias for `core::result::Result<CallResponse, CallError>`.

## Declarative Macros

- `router_builder!(handlers: [...], resources: [...])`  
  Macro to register multiple handlers and optional resources in one call.
  Also supports `router_builder!(handler1, handler2)` and `router_builder!(handlers: [...])`.

- `resources_builder!(x1, x2, …)`  
  Macro to build a `ResourcesBuilder` from a list of values.

## Derive (Proc) Macros

- `#[derive(RpcHandlerError)]`  
  Implements `IntoHandlerError` for an error enum/struct. Requires `Debug + Send + Sync + 'static`.

- `#[derive(RpcParams)]`  
  Implements `IntoParams` for a `serde::Deserialize` type. Requires `DeserializeOwned + Send`.

- `#[derive(RpcResource)]`  
  Implements `FromResources` for a type. Requires `Clone + Send + Sync + 'static`.

