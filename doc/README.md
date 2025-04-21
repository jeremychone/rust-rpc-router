# rpc-router

**rpc-router** is a simple, light, type-safe, and macro-free JSON-RPC router library designed for Rust asynchronous applications.

It provides a flexible way to define and route JSON-RPC requests to corresponding handler functions, managing resources, parameters, and error handling efficiently.

**Key Features:**

- **Type-Safe Handlers:** Define RPC handlers as regular async Rust functions with typed parameters derived from request resources and JSON-RPC parameters.
- **Resource Management:** Inject shared resources (like database connections, configuration, etc.) into handlers using the `FromResources` trait.
- **Parameter Handling:** Automatically deserialize JSON-RPC `params` into Rust types using the `IntoParams` trait. Supports optional parameters and default values via `IntoDefaultRpcParams`.
- **Composable Routers:** Combine multiple routers using `RouterBuilder::extend` for modular application design.
- **Error Handling:** Robust error handling with distinct types for routing errors (`rpc_router::Error`) and handler-specific errors (`HandlerError`), allowing applications to retain and inspect original error types.
- **JSON-RPC Response Types:** Provides `RpcResponse`, `RpcError`, and `RpcResponseParsingError` for representing and parsing standard JSON-RPC 2.0 responses, with direct conversion from router `CallResult`.
- **Ergonomic Macros (Optional):** Includes helper macros like `router_builder!` and `resources_builder!` for concise router and resource setup (can be used without macros as well).
- **Minimal Dependencies:** Core library has minimal dependencies (primarily `serde`, `serde_json`, and `futures`).

## Core Concepts

1.  **`Router`:** The central component that holds RPC method routes and associated handlers. It's built using `RouterBuilder`.
2.  **`Handler` Trait:** Implemented automatically for async functions matching specific signatures. Handles resource extraction, parameter parsing, and execution logic.
3.  **`Resources`:** A type map holding shared application state (e.g., database pools, config). Handlers access resources via types implementing `FromResources`.
4.  **`FromResources` Trait:** Types implementing this trait can be injected as parameters into RPC handlers, fetched from the `Resources` map. Use `#[derive(RpcResource)]` for convenience.
5.  **`IntoParams` Trait:** Types implementing this trait can be used as the final parameter in an RPC handler to receive and deserialize the JSON-RPC `params` field. Use `#[derive(RpcParams)]` for simple deserialization or implement the trait manually for custom logic.
6.  **`CallResult`:** The `Result` type returned by `router.call(...)`, containing either `CallSuccess` (on success) or `CallError` (on failure). Includes the original `RpcId` and method name for context.
7.  **`RpcResponse`:** Represents a standard JSON-RPC 2.0 response object (success or error). Can be easily created from a `CallResult`.
8.  **`HandlerError`:** A wrapper for application-specific errors returned by handlers, allowing the application layer to recover the original error type. Use `#[derive(RpcHandlerError)]` on your custom error enums.

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
    // ... database connection pool
    url: String, // Example field
}

// --- Define Params
#[derive(Deserialize, Serialize, RpcParams)]
struct HelloParams {
    name: String,
}

// --- Define Custom Error (Optional)
#[derive(Debug, thiserror::Error, rpc_router::RpcHandlerError)]
enum MyHandlerError {
    #[error("Something went wrong: {0}")]
    SpecificError(String),
}


// --- Define RPC Handlers
async fn hello(state: AppState, params: HelloParams) -> HandlerResult<String> {
    // Use injected resources and parsed params
    Ok(format!("Hello {}, from app version {}!", params.name, state.version))
}

async fn get_db_url(db: DbPool) -> HandlerResult<String> {
    Ok(db.url.clone())
}

// Example handler returning a custom error
async fn might_fail() -> HandlerResult<i32> {
    // Simulate an error condition
    if rand::random() { // Requires `rand` crate
       Err(MyHandlerError::SpecificError("Random failure".to_string()))
    } else {
       Ok(42)
    }
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
        handlers: [hello, get_db_url, might_fail]
    ].build();

    // --- Simulate an RPC Call
    let request_json = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "hello",
        "params": {"name": "World"}
    });
    let rpc_request = RpcRequest::from_value(request_json)?;

    // Execute the call using the router's base resources
    let call_result = router.call_with_resources(rpc_request, resources).await;

    // --- Process the Result
    let rpc_response = RpcResponse::from(call_result); // Convert CallResult to RpcResponse

    // Serialize the response (e.g., to send back to the client)
    let response_json = serde_json::to_string_pretty(&rpc_response)?;
    println!("JSON-RPC Response:\n{}", response_json);

    // --- Example: Inspecting a specific error
    let fail_request_json = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "might_fail",
        "params": null // Or omit params if handler doesn't take them
    });
     let fail_rpc_request = RpcRequest::from_value(fail_request_json)?;
     let fail_call_result = router.call(fail_rpc_request).await; // Using router's own resources

     if let Err(call_error) = fail_call_result {
         // Access the router's error type
         if let rpc_router::Error::Handler(handler_error) = call_error.error {
             // Try to downcast to your specific handler error
             if let Some(my_error) = handler_error.get::<MyHandlerError>() {
                 println!("Caught specific handler error: {:?}", my_error);
                 match my_error {
                     MyHandlerError::SpecificError(msg) => {
                        println!("Specific error message: {}", msg);
                     }
                 }
             } else {
                  println!("Caught a handler error, but not MyHandlerError: {}", handler_error.type_name());
             }
         } else {
              println!("Caught a router error: {:?}", call_error.error);
         }
     }


    Ok(())
}
```

*(Note: The example uses `tokio` for the async runtime and `thiserror` for simplified error definition, but `rpc-router` itself is runtime-agnostic and doesn't require `thiserror`)*.

## Learn More

- [**API Documentation (`doc/all-apis.md`)**](./all-apis.md): Detailed explanation of all public types, traits, and functions.
- [**Examples (`/examples`)**](../examples): Practical code examples demonstrating various features.

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

Licensed under either of [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0) or [MIT license](http://opensource.org/licenses/MIT) at your option.

