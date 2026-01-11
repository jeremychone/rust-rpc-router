# rpc-router API Reference for LLM

## Core Request/Response Types

### RpcId
Enum for JSON-RPC 2.0 IDs.
- `String(Arc<str>)`
- `Number(i64)`
- `Null`
Constructors: `new_uuid_v4()`, `new_uuid_v7()`, `from_scheme(IdSchemeKind, IdSchemeEncoding)`.

### RpcRequest
Primary request structure for methods expecting a response.
- `id: RpcId`
- `method: String`
- `params: Option<serde_json::Value>`
Methods: `from_value(Value)`, `from_value_with_checks(Value, RpcRequestCheckFlags)`.

### RpcNotification
Request structure for notifications (no response).
- `method: String`
- `params: Option<serde_json::Value>`
Methods: `from_value(Value)`.

### RpcResponse
Enum representing the final JSON-RPC response.
- `Success(RpcSuccessResponse)`
- `Error(RpcErrorResponse)`
Methods: `id()`, `is_success()`, `is_error()`, `into_parts()`.
Conversions: `From<CallResult>`, `From<CallSuccess>`, `From<CallError>`.

### RpcError
The JSON-RPC 2.0 Error Object.
- `code: i64`
- `message: String`
- `data: Option<Value>`
Predefined codes: `CODE_PARSE_ERROR (-32700)`, `CODE_INVALID_REQUEST (-32600)`, `CODE_METHOD_NOT_FOUND (-32601)`, `CODE_INVALID_PARAMS (-32602)`, `CODE_INTERNAL_ERROR (-32603)`.

## Router and Resources

### Router
The main executor. Usually wrapped in `Arc`.
- `call(RpcRequest)`: Exec with router's base resources.
- `call_with_resources(RpcRequest, Resources)`: Exec with additional overlaid resources.
- `call_route(id, method, params)`: Lower level call.

### RouterBuilder
- `append(name, handler_fn)`: Generic add.
- `append_dyn(name, handler_fn.into_dyn())`: Type-erased add (recommended for large routers).
- `append_resource(val)`: Add base resource to all calls.
- `extend(other_builder)`: Merge routes and resources.
- `build()`: Returns `Router`.

### Resources
Type-safe container for shared state.
- `Resources::builder().append(T).build()`
- `get<T>()`: Returns `Option<T>`. Looks in overlay then base.

## Traits for Handlers

### IntoParams
Used for the last argument of a handler function.
- Implement for structs to be used as `params`.
- Default impl uses `serde_json::from_value`.

### FromResources
Used for resource injection arguments (0 to 8).
- Implement for types to be fetched from the `Resources` map.

### IntoHandlerError
Allows application errors to be converted into `HandlerError`.
- `HandlerError` holds the original error in an `Any` map.
- Retrieve original error with `handler_error.get::<T>()` or `remove::<T>()`.

## Macros and Derives

### Macros
- `router_builder![handlers: [...], resources: [...]]`
- `resources_builder![...]`

### Derives
- `#[derive(RpcParams)]`: Implements `IntoParams`.
- `#[derive(RpcResource)]`: Implements `FromResources`.
- `#[derive(RpcHandlerError)]`: Implements `IntoHandlerError` and `std::error::Error`.

## Error Types
- `rpc_router::Error`: Routing level errors (MethodUnknown, ParamsParsing, etc.).
- `CallError`: Contextual error containing `RpcId`, `method`, and `rpc_router::Error`.
- `RpcRequestParsingError`: Validation errors during request parsing.
