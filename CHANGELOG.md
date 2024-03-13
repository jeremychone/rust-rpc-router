`.` minor | `*` Major | `+` Addition | `^` improvement | `!` Change

> For the `0.1.x` releases, there may be some changes to types or API naming. Therefore, the version should be locked to the latest version used, for example, `=0.1.0`. I will try to keep changes to a minimum, if any, and document them in the future [CHANGELOG](CHANGELOG.md).
>
> Once `0.2.0` is released, I will adhere more strictly to the semantic versioning methodology.


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

