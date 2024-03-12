`.` minor | `*` Major | `+` Addition | `^` improvement | `!` Change | 


## 2024-03-12 - `0.1.1`

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

