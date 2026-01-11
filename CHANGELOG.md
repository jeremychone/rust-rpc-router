`.` minor | `*` Major | `+` Addition | `^` improvement | `!` Change

## 2026-01-02 - `0.2.0`

- `!` API RENAME - CallSuccess now (from CallResponse)
- `!` API RENAME - Request is now RpcRequest (because it represents core json-rpc construct)
- `!` RpcId - Refactor Rpc Id to be its own type
- `+` RpcResponse - Add RpcResponse construct
- `+` RpcRequest - added from_value_with_checks to allow to skip version and/or id checks
- `^` router - implement FromResources for Router
- `^` RpcRequest - add from_value_with_checks
- `^` export the RpcHandlerWrapperTrait
- `^` add RpcRequest new
- `^` RpcRequest - add custom serializer
- `.` doc - add doc/*.md documentations
- `.` doc - update doc/ markdowns

## 2024-03-14 - `0.1.3`

- `+` add `RouterBuilder::extend_resources(..)`
- `!` rename `RouterBuilder::set_resources_builder(..)` to `RouterBuilder::set_resources(..)`

## 2024-03-13 - `0.1.2`

- `^` Add `IntoHandlerError` for `String`, `Value`, and `&'static str`.
- `^` Add HandlerError::new:<T>()
- `.` remove `std::error::Error` error requirement for HandlerError .error

## 2024-03-12 - `0.1.1`

> Note: `v0.1.1` changes from `0.1.0`
> - `router.call(resources, request)` was **renamed** to `router.call_with_resources(request, resources)`. 
> - Now, the Router can have its own resources, enabling simpler and more efficient sharing of common resources across calls, 
> while still allowing custom resources to be overlaid at the call level.
> - `router.call(request)` uses just the default caller resources.
>
> See [CHANGELOG](CHANGELOG.md) for more information. 
> 
> [Rust10x rust-web-app](https://github.com/rust10x/rust-web-app) has been updated. 

- `!` Changed `router.call(resources, request)` to `router.call_with_resources(request, resources)`.
- `+` `Router` can now have base/common resources that are "injected" into every call.
  - Use `router_builder.append_resource(resource)...` to add resources.
  - The method `router.call_with_resources(request, resources)` overlays the call resources on top of the router resources.
  - `router.call(request)` uses only the Router resources.
- `^` `router_builder!` macro now allows building the route and resource.

```rust
let rpc_router = router_builder!(
	handlers: [get_task, create_task],         // will be turned into routes
	resources: [ModelManager {}, AiManager {}] // common resources for all calls
)
.build();
```

## 2024-03-11 - `0.1.0`

- `*` Initial

