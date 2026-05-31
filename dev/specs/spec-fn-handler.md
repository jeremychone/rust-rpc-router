# rpc-router Pattern Specification

## Intent

This specification captures the core architectural patterns and interface contracts used by the `rpc-router` library, in a language‑agnostic manner, so that other programs can reuse them to implement similar JSON‑RPC routing, resource injection, parameter handling, and error adaptation.

The intended patterns include:

- **Registration and dispatch**: A builder pattern for constructing an immutable shared router that maps method names to dynamically dispatched handler wrappers, supporting handler discovery and execution.
- **Handler abstraction**: A normalised callable contract for async handlers that accept typed resources and optional typed parameters, producing serialisable results. Concrete handler functions are adapted to this contract, allowing different signatures to coexist.
- **Resource injection**: Typed state management where handler dependencies are fetched from a shared, immutable type map. A per‑call overlay mechanism allows overriding or extending router‑wide resources for individual requests.
- **Params handling**: Conversion of optional JSON‑RPC parameters into strong types, with fallback behaviour for missing parameters (error or default).
- **Error model**: A layered error system separating library‑level routing and dispatch errors from application‑level handler errors, preserving type information through a type‑erasing wrapper.
- **Request and response boundary**: Typed JSON‑RPC message structures (`Request`, `Notification`, `Response`, `Error`) with intermediate typed call results that preserve request context before final serialisation.
- **Composability**: Builder composition and resource builder merging with override semantics, enabling modular construction of routers and resource sets.

The scope is limited to the library core: no transport integration (HTTP, WebSocket, CLI, GUI) is specified. The patterns are presented as a set of traits, types, and builder interfaces that together implement a type‑safe, concurrent JSON‑RPC dispatch framework.

## Code Design

### Handler registration and dispatch

The registration and dispatch pattern uses a builder–built immutable router with dynamic dispatch for handler storage.

- **Router builder**: A `RouterBuilder` accumulates handler entries (method name → handler wrapper) and optional base resources. Once built, the router is immutable and can be shared across concurrent callers.
- **Dynamic dispatch**: Handlers with different signatures are type-erased into a common trait object, typically `Box<dyn RpcHandlerWrapperTrait>`, and stored in a map keyed by the JSON‑RPC method name.
- **Internal map**: `RouterInner` holds a `HashMap<&'static str, Box<dyn RpcHandlerWrapperTrait>>` that maps method names to their wrapper.
- **Convenience macros** (such as `router_builder!`) provide ergonomic ways to register multiple handlers and resources in a single expression without changing the underlying dispatch model.

Registration flow (pseudo‑code):

```rust
// build phase
let builder = RouterBuilder::default();
builder = builder.append_dyn("method_name", handler_fn.into_dyn());   // dynamic add
// or
builder = builder.append("method_name", handler_fn);                 // monomorphised add
// optional resources
builder = builder.append_resource(some_resource);
let router: Router = builder.build();   // immutable, Arc‑wrapped internals
```

Dispatch flow:

```rust
// call phase
let rpc_request = RpcRequest { id, method: "method_name", params };
let call_result: CallResult = router.call(rpc_request).await;
// Router resolves method → wrapper → handler → async execution → serialised Value or Error
```

The pattern supports composability through `RouterBuilder::extend`, enabling merging of route tables and resources from multiple builder instances.


### Handler abstraction and implementation

The handler abstraction defines a unified callable contract that all async handler functions must conform to, regardless of their concrete signature. A runtime wrapper type erases the concrete handler so it can be stored and dispatched uniformly by the router.

- **Handler trait** – A generic trait parameterised by resource inputs (`T`), params input (`P`), and result type (`R`). It declares an associated async output `Future` that yields a serialisation‑ready result, and provides a method to convert the handler into a boxed dynamic wrapper.
- **Normalised signature** – A single `call` method receives immutable resources and an optional params value, extracts typed inputs, invokes the user‑provided async function, and maps both success and error outcomes into a uniform result shape.
- **Dynamic wrapper** – A concrete wrapper type stores a cloneable handler and implements a trait‑object‑safe interface (`HandlerWrapperTrait`). This type‑erasing wrapper is what the router stores and invokes.
- **Generated implementations** – A macro generates `Handler` implementations for functions taking zero to eight resource arguments, with or without a params argument. This avoids manual re‑implementation for each arity while maintaining type safety.

Pseudo‑code for the handler trait and wrapper:

```rust
trait Handler<T, P, R> : Clone {
    type Future: Future<Output = Result<Value>>;
    fn call(resources: Resources, params: Option<Value>) -> Self::Future;
    fn into_dyn() -> Box<dyn HandlerWrapperTrait>;
}

// Type‑erasing wrapper
struct HandlerWrapper<H, T, P, R> {
    handler: H,
}
impl HandlerWrapperTrait for HandlerWrapper<H, T, P, R> { … }

// Macro‑generated impls for different arities, e.g.
// impl Handler<(T1, T2), P, R> for fn(T1, T2, P) -> Fut { … }
// impl Handler<(T1, T2), (), R> for fn(T1, T2) -> Fut { … }
```

All handler functions are ultimately stored as `Box<dyn HandlerWrapperTrait>` in the router's method map, enabling uniform dispatch.


### Resource injection and management

The resource subsystem provides type‑safe dependency injection for handler functions. Handlers declare resources they need as arguments, and the router fulfills them from an immutable `Resources` container. A per‑call overlay mechanism allows call‑specific overrides without mutating router‑wide state.

- **`FromResources` trait** – Implemented by types that can be retrieved from the resource container. The default implementation fetches the value by static type via `resources.get::<Self>()`. A blanket implementation exists for `Option<T>`, allowing handlers to declare optional dependencies.
- **`Resources` container** – An immutable, `Clone`‑friendly container holding two layers of typed data: a shared *base* layer (`Arc<ResourcesInner>`) and an *overlay* layer. Lookup checks the overlay first, then falls back to the base.
- **`ResourcesBuilder`** – A mutable staging builder that accumulates typed values, supports type‑based retrieval before finalisation, and can be merged with other builders to produce the final `Resources` object via `build()`.
- **Router‑level resource attachment** – `RouterBuilder::append_resource(...)` adds a resource to the router’s base `Resources`. All calls that use the router will have access to these router‑wide resources.
- **Per‑call overlay** – `Router::call_with_resources(...)` accepts an additional `Resources` object that becomes the overlay for that single invocation. The overlay overrides or supplements base resources for the duration of the call. The effective resources are computed by layering the per‑call `Resources` on top of the router’s base `Resources`.

Pseudo‑code for resource injection:

```rust
// Build router with base resources
let router = Router::builder()
    .append_resource(db_pool)
    .append_resource(config)
    .append("method", handler_fn)
    .build();

// Handler signature example
async fn handler_fn(pool: DbPool, cfg: Config, params: MyParams) -> Result<Value> { … }

// Call with additional per‑call resource
let extra_resources = Resources::builder().append(req_id).build();
let result = router.call_with_resources(request, extra_resources).await;
```

The lookup order (overlay then base) allows temporary overrides, such as replacing a global request‑scoped value with a request‑specific one, without affecting other calls.


### Params handling

The params subsystem controls how the optional JSON‑RPC `params` value is converted into a strongly‑typed Rust argument for handler functions.

- **`IntoParams` trait** – Defines a method that takes `Option<Value>` and returns a typed instance. The **default implementation** deserialises a `Some(Value)` via `serde_json::from_value`; when the value is `None`, it returns a dedicated `ParamsMissingButRequested` error. This ensures that handlers that expect parameters fail fast when none are provided.
- **`IntoDefaultRpcParams` marker trait** – When a type implements both `DeserializeOwned + Send + Default` and this marker trait, a blanket `IntoParams` implementation is provided that calls `Default::default()` when the incoming params are `None`. This is useful for handlers with optional parameters.
- **Blanket implementations** – Additional blanket `IntoParams` impls cover common convenience cases:
  - `Option<D>` where `D: IntoParams`: allows a handler param to be `Option<MyParams>`, treating a missing params value as `None` and a present value as `Some(parsed)`.
  - `serde_json::Value`: passthrough that accepts the raw JSON value directly, useful for handlers that need unstructured data.
- **Derive macro** – `#[derive(RpcParams)]` generates a trivial `IntoParams` impl that relies on the default behaviour, reducing boilerplate for simple structs.

Pseudo‑code for the trait and marker trait:

```rust
trait IntoParams {
    fn into_params(value: Option<Value>) -> Result<Self>;
}
trait IntoDefaultRpcParams: Deserialize + Send + Default {}

// Blanket impl for IntoDefaultRpcParams
impl<P> IntoParams for P where P: IntoDefaultRpcParams {
    fn into_params(value: Option<Value>) -> Result<Self> {
        match value {
            Some(v) => deserialize(v),
            None => Ok(Self::default()),
        }
    }
}
```

This design keeps params handling explicit and flexible: most handlers use a simple derive or manual impl, while complex cases can opt into defaults, optional wrappers, or raw JSON passthrough.

### Error model and adaptation

The error model separates library‑level routing and dispatch errors from application‑specific handler errors. A library‑level error type categorises failures into distinct variants, while a type‑erasing wrapper preserves application error types without requiring a global error enum.

- **Library error enumeration** – A top‑level `Error` enum (e.g. `rpc_router::Error`) collects all failures that can arise during routing, parameter parsing, resource extraction, handler result serialisation, and handler adaptation. Each variant carries structured information appropriate to the failure mode.
- **Handler error wrapper (`HandlerError`)** – A type‑erased container that stores an arbitrary application error value (any type) together with its type name. It supports both immutable inspection (`get::<T>()`) and removal (`remove::<T>()`) to allow callers to downcast and handle specific application errors when desired.
- **`IntoHandlerError` trait** – Defines a conversion from an application error type into a `HandlerError`. Blanket or default implementations exist for common types (`String`, `&str`, `serde_json::Value`), and a derive macro (`RpcHandlerError`) provides a trivial implementation for user error types. Handlers return their native error type, which is automatically wrapped into a `HandlerError` before being embedded in the library error.
- **Separation of concerns** – The library error captures operational failures; the inner `HandlerError` acts as a carrier for domain‑specific errors without imposing any constraints on the application’s error design. Callers at the integration boundary can inspect the library error variant and, when a `Handler` variant is encountered, attempt to extract the concrete application error.

Pseudo‑code overview:

```rust
// Library error
enum Error {
    ParamsParsing(serde_json::Error),
    ParamsMissingButRequested,
    MethodUnknown,
    FromResources(ResourceError),
    HandlerResultSerialize(serde_json::Error),
    Handler(HandlerError),
}

// Type‑erased handler error
struct HandlerError {
    // container with TypeId key, type_name stored
}

// Conversion trait
trait IntoHandlerError {
    fn into_handler_error(self) -> HandlerError;
}

// Derive macro for trivial impl
#[derive(RpcHandlerError)]
impl IntoHandlerError for MyAppError {}
```

### Request and response boundary

The library defines typed JSON‑RPC message structures that model requests, notifications, responses, and error objects, along with intermediate call‑result types that preserve request context before final serialisation.

- **JSON‑RPC message types**: `RpcRequest` holds an `id`, `method`, and optional `params`. `RpcNotification` is a request without an `id`. `RpcResponse` is an enum with `Success` or `Error` variants, each containing an `id` and either a result value or an `RpcError` object. `RpcError` carries a numeric error code, a message, and optional data. `RpcId` supports string, number, or null IDs.
- **Intermediate call‑result types**: `CallResult` is a `Result<CallSuccess, CallError>`. `CallSuccess` bundles the request `id`, `method`, and a successful `Value` result. `CallError` bundles the `id`, `method`, and the library‑level `Error` that caused the failure. These types decouple routing from the final JSON‑RPC response shape and allow tracing/logging with the original method context.
- **Boundary conversions**: `RpcResponse` can be constructed from a `CallResult`, a `CallSuccess`, or a `CallError`. This conversion maps routing outcomes into the final JSON‑RPC protocol representation, preserving the request ID and adding the `"jsonrpc": "2.0"` envelope.

Pseudo‑code outline:

```rust
// JSON‑RPC message types
struct RpcRequest {
    id: RpcId,
    method: String,
    params: Option<Value>,
}

struct RpcNotification {
    method: String,
    params: Option<Value>,
}

enum RpcResponse {
    Success(RpcSuccessResponse),
    Error(RpcErrorResponse),
}

struct RpcSuccessResponse {
    id: RpcId,
    result: Value,
}

struct RpcErrorResponse {
    id: RpcId,
    error: RpcError,
}

struct RpcError {
    code: i64,
    message: String,
    data: Option<Value>,
}

// Intermediate routing results
type CallResult = Result<CallSuccess, CallError>;

struct CallSuccess {
    id: RpcId,
    method: String,
    value: Value,
}

struct CallError {
    id: RpcId,
    method: String,
    error: LibraryError,
}

// Conversion from routing to protocol response
impl From<CallResult> for RpcResponse { … }

```


### Extensibility and composition

The router and resource subsystems support merging and composition through builder extension, allowing modular construction of routers and shared resource sets.

- **Router builder extension** – `RouterBuilder::extend(other_builder)` consumes another `RouterBuilder` and merges its route table and base resources into the current builder. This enables multiple modules to independently define handlers and resources, then combine them into a single router.
- **Resources builder extension** – `ResourcesBuilder::extend(other_builder)` merges the resource maps from another builder. When the same resource type exists in both builders, the value from `other_builder` overrides the existing one, providing deterministic override semantics.
- **Router as a resource** – `Router` implements `FromResources`, allowing handlers to request the router instance itself as an injected resource. This is useful for handlers that need to perform recursive calls or dispatch to other methods.

Pseudo‑code for composability:

```rust
// Building from separate modules
let module_a = RouterBuilder::default()
    .append_resource(db_pool)
    .append("a.method", handler_a);

let module_b = RouterBuilder::default()
    .append_resource(logger)
    .append("b.method", handler_b);

// Merge route tables and resources
let router = module_a.extend(module_b).build();

// Handlers can receive the router as a resource
async fn handler_a(router: Router, params: MyParams) -> Result<Value> {
    // Use router to call another method
    let result = router.call_route(…).await;
    // …
}
```


## Design Considerations

- **Immutable shared router**: The router is constructed once and then shared across concurrent callers via `Arc`, avoiding locking and enabling safe re-use in async execution contexts.
- **Dynamic dispatch for handler storage**: Using `Box<dyn HandlerWrapperTrait>` in the route table allows handlers with different signatures to coexist in a single router, at the cost of a small dynamic dispatch overhead per call.
- **Type-based resource extraction**: Handlers declare dependencies by Rust type, which keeps handler signatures idiomatic. The type map provides automatic wiring without string keys. The trade-off is that only one resource of a given type can be stored at a time.
- **Resource overlay for per-call customization**: The base + overlay model allows per-request overrides without mutating router state or rebuilding. This supports patterns like request-scoped values while keeping the router thread-safe.
- **Single optional params input**: Aligns with JSON-RPC semantics and avoids complex multi-param decomposition. Handlers that need multiple values can use a struct that implements `Deserialize`.
- **Serialization boundary**: `serde_json::Value` is used as the normalized input and output format, keeping the router transport-agnostic and compatible with any JSON-RPC wire protocol.
- **Separation of library and application errors**: The `Error` enum distinguishes routing/adaptation failures from domain-specific handler errors. `HandlerError` boxes the application error, avoiding a global enum while still allowing downcast extraction.
- **Flattened public API**: The crate re-exports items from sub-modules so common usage is concise, while implementation details remain organized by responsibility.
- **Derive macros as optional helpers**: Procedural macros generate trait implementations for common patterns, but the design remains trait-oriented so manual implementations are always possible.
