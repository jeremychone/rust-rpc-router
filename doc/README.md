# rpc-router Documentation

## Introduction

`rpc-router` is a lightweight, asynchronous JSON-RPC routing library for Rust. It allows you to define RPC handlers as simple `async fn` functions, manage shared resources (like database connections or configuration), handle request parameters, and manage application-specific errors cleanly.

The core goal is to provide a type-safe and ergonomic way to build JSON-RPC APIs without excessive boilerplate.

## Core Concepts

-   **JSON-RPC 2.0**: The library processes requests adhering to the JSON-RPC 2.0 specification, expecting `jsonrpc: "2.0"`, `method`, `id`, and optional `params`.
-   **`RpcId`**: Represents the JSON-RPC request ID, which can be a String, Number, or Null. Ensures type safety for IDs.
-   **`RpcRequest`**: Represents a validated JSON-RPC 2.0 request, containing a type-safe `RpcId`, method name, and optional parameters. Parsing is done via `RpcRequest::from_value` or `TryFrom<Value>`.
-   **Routing**: The `Router` maps incoming request `method` names to specific handler functions.
-   **Handlers**: Handlers are `async fn` functions that contain your application logic. They receive necessary resources and request parameters as arguments.
-   **Resources**: These represent shared state or dependencies (e.g., database connections, configuration objects, service clients) that handlers need. Resources are managed by the `Router` and accessed by handlers type-safely.
-   **Parameters (`params`)**: The JSON `params` field in the request is automatically deserialized into a specific Rust type defined by the handler function using the `IntoParams` trait.
-   **Error Handling**: Handlers return a `Result<T, AppError>`, where `AppError` is your application-specific error type implementing `IntoHandlerError`. The library wraps this in its own error types (`HandlerError`, `rpc_router::Error`) to provide context while allowing you to easily extract your original error.

## Key Constructs

Here are the central types and traits you'll interact with:

-   **`Router`**:
    -   The main entry point for handling requests.
    -   Holds the registered method routes and common resources.
    -   Created using `Router::builder()` or the `router_builder!` macro.
    -   Dispatches requests via `.call(request: RpcRequest)` or `.call_with_resources(request: RpcRequest, additional_resources: Resources)`.

    ```rust
    // Conceptual usage
    let router: Router = /* ... build router ... */;
    let rpc_request: RpcRequest = /* ... parse request value ... */.try_into()?;
    let result: CallResult = router.call(rpc_request).await;
    ```

-   **`RouterBuilder`**:
    -   Used to configure and build a `Router`.
    -   Methods:
        -   `.append(name, handler)`: Add a handler function (generic).
        -   `.append_dyn(name, handler.into_dyn())`: Add a handler function (dynamic dispatch, preferred).
        -   `.append_resource(resource)`: Add a resource available to all handlers managed by this router.
        -   `.extend(other_builder)`: Merge another builder's routes and resources.
        -   `.build()`: Consumes the builder and returns a `Router`.

    ```rust
    // Conceptual usage
    let builder = Router::builder()
        .append_dyn("get_user", get_user.into_dyn())
        .append_resource(DatabaseConnection::new())
        .append_resource(Config::load());
    let router = builder.build();
    ```

-   **`router_builder!` macro**:
    -   A convenient macro for creating a `RouterBuilder`.

    ```rust
    // Handlers only
    let builder = router_builder!(handler_one, handler_two);

    // Handlers and resources
    let builder = router_builder!(
        handlers: [handler_one, handler_two],
        resources: [MyResource1 {}, MyResource2 {}]
    );
    ```

-   **`Handler` trait**:
    -   The trait that defines an RPC handler function.
    -   You **don't** typically implement this manually. It's automatically implemented for `async fn` functions with a specific signature:

        ```rust
        async fn my_handler(
            resource1: ResourceType1, // 0..N args implementing FromResources
            resource2: ResourceType2,
            // ...
            params: ParamsType // Optional last arg implementing IntoParams
        ) -> Result<ReturnValue, AppError> // ReturnValue: Serialize, AppError: IntoHandlerError
        where
             ResourceType1: FromResources,
             ResourceType2: FromResources,
             ParamsType: IntoParams,
             ReturnValue: Serialize,
             AppError: IntoHandlerError + Send + Sync + 'static, // Note: AppError needs Send + Sync + 'static
        {
            // ... logic ...
        }
        ```

    -   The `resource` arguments are resolved from the `Router`'s resources or call-specific resources.
    -   The optional `params` argument is deserialized from the request's `params` field.
    -   The `Result`'s `Ok` value must be `serde::Serialize`.
    -   The `Result`'s `Err` value must implement `IntoHandlerError` and be `Send + Sync + 'static`.

-   **`Resources`**:
    -   A type map holding instances of shared state/dependencies.
    -   Created using `Resources::builder()`.
    -   `Router` holds base resources; `call_with_resources` allows overlaying call-specific resources.

```rust
// Conceptual usage
let call_resources = Resources::builder()
    .append(UserContext { user_id: 123 })
    .build();
```
-   **`FromResources` trait & `#[derive(RpcResource)]`**:
    -   Trait allowing a type to be requested from `Resources` by a handler.
    -   Implement manually or derive using `#[derive(Clone, RpcResource)]`.
    -   The `#[derive(RpcResource)]` automatically implements `FromResources` and requires `Clone + Send + Sync + 'static`.

    ```rust
    // Using derive (recommended)
    #[derive(Clone, RpcResource)]
    struct MyDbConnection { /* ... */ }

    // Manual implementation (less common)
    #[derive(Clone)]
    struct MyConfig { /* ... */ }
    impl FromResources for MyConfig { /* usually default impl is fine */ }
    ```

-   **`IntoParams` trait & `#[derive(RpcParams)]`**:
    -   Trait allowing a type to be deserialized from the request's `params` field (`Option<serde_json::Value>`).
    -   Implement manually or derive using `#[derive(Deserialize, RpcParams)]`.
    -   The `#[derive(RpcParams)]` automatically implements `IntoParams` using `serde::Deserialize`. It requires the `params` field to be present in the request (unless the type implements `IntoDefaultRpcParams`) and the type must be `DeserializeOwned + Send`.

    ```rust
    use serde::Deserialize;
    use rpc_router::RpcParams; // Import the derive macro

    // Using derive (recommended)
    #[derive(Deserialize, RpcParams)]
    struct TaskCreateParams {
        title: String,
        priority: Option<u32>,
    }

    // Manual implementation (for custom logic, less common)
    #[derive(Deserialize)]
    struct GetByIdParams { id: i64 }
    impl IntoParams for GetByIdParams {} // Default impl uses serde
    ```

-   **`IntoHandlerError` trait & `#[derive(RpcHandlerError)]`**:
    -   Trait allowing an application error type to be converted into a `HandlerError`. This enables the router to handle errors generically while allowing the caller to potentially extract the specific error type later.
    -   Implement manually or derive using `#[derive(RpcHandlerError)]`.
    -   The derive macro requires the type to implement `Debug` and `Send + Sync + 'static`. It's often used with helper crates like `thiserror` or `derive_more::Display`.

    ```rust
    use thiserror::Error; // Example using thiserror
    use rpc_router::RpcHandlerError; // Import the derive macro

    // Using derive (recommended)
    #[derive(Debug, Error, RpcHandlerError)]
    pub enum MyAppError {
        #[error("Item not found with id: {0}")]
        NotFound(i64),
        #[error("Invalid input: {0}")]
        InvalidInput(String),
        #[error("Database error: {0}")]
        RepoError(String), // Assuming DB error converts to String
    }

    // Manual implementation (less common)
    #[derive(Debug)] // Debug is needed for HandlerError wrapper
    struct LegacyError(String);
    impl std::fmt::Display for LegacyError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "Legacy: {}", self.0) } }
    impl std::error::Error for LegacyError {}
    // Manual impl requires Send + Sync + 'static
    impl IntoHandlerError for LegacyError {}
    ```

-   **`HandlerError`**:
    -   An opaque wrapper around your application error (`AppError`). It's part of `rpc_router::Error::Handler`.
    -   Provides methods like `.get::<AppError>()` or `.remove::<AppError>()` to attempt to downcast and retrieve the original error.

-   **`RpcRequest`**:
    -   Represents a parsed and validated JSON-RPC 2.0 request (`{ "jsonrpc": "2.0", "id": ..., "method": ..., "params": ... }`).
    -   Contains `id: RpcId`, `method: String`, `params: Option<Value>`.
    -   Typically created via `RpcRequest::from_value(value)?` or `value.try_into()?`. These methods perform JSON-RPC 2.0 validation (checking `jsonrpc: "2.0"`, presence/type of `method` and `id`) and parse the `id` into the type-safe `RpcId`. Fails with `RpcRequestParsingError`.

-   **`RpcId`**:
    -   Enum representing a valid JSON-RPC ID: `String(Arc<str>)`, `Number(i64)`, or `Null`.
    -   Ensures type safety and efficient cloning for request IDs. Parsed from `Value` during `RpcRequest` creation.

-   **`CallResponse` / `CallError`**:
    -   The `Result` type returned by `router.call(...)`, wrapping either success (`CallResponse`) or failure (`CallError`).
    -   **Important**: These are *not* the final JSON-RPC response/error objects. They contain the necessary information (`id: RpcId`, `method`, `value`/`error`) for you to construct the actual JSON-RPC response.
    -   `CallResponse { id: RpcId, method: String, value: Value }`: Contains the request ID, method name, and the serialized return value from the handler.
    -   `CallError { id: RpcId, method: String, error: rpc_router::Error }`: Contains the request ID, method name, and the router/handler error.

-   **`rpc_router::Error`**:
    -   The enum representing errors that can occur during routing or handler execution *within* the `rpc-router` library itself. Variants include:
        -   `MethodUnknown`: The requested method wasn't found.
        -   `ParamsParsing`: Failed to deserialize `params` into the handler's expected type.
        -   `ParamsMissingButRequested`: The handler expected `params` (and type doesn't impl `IntoDefaultRpcParams`), but none were provided.
        -   `FromResources(FromResourcesError)`: Failed to retrieve a requested resource.
        -   `HandlerResultSerialize`: Failed to serialize the handler's successful return value.
        -   `Handler(HandlerError)`: An error occurred within the handler itself (this wraps your `AppError`).

-   **`RpcRequestParsingError`**:
    - The enum representing errors during the parsing and validation of the initial JSON `Value` into an `RpcRequest`. Variants detail issues like missing/invalid `jsonrpc` version, missing/invalid `method`, or missing/invalid `id`.

## Usage Examples

### 1. Minimal Example

```rust
// file: examples/01-minimal.rs
use rpc_router::{IntoParams, RpcRequest, Resources, Router, HandlerResult, RpcId, RpcParams, CallResult}; // Added RpcId, RpcParams, CallResult
use serde::Deserialize;
use serde_json::{json, Value};

// Define parameters type (must impl IntoParams)
// Using derive is simpler. RpcParams requires Deserialize.
#[derive(Deserialize, RpcParams)]
struct EchoParams {
    message: String,
}

// Define the handler function
// Takes no resources, only params. Returns Result<Value, HandlerError>
// HandlerResult<T> is shorthand for Result<T, HandlerError>
// We use HandlerError directly here for simplicity (no custom error).
// The return type must be Serialize.
async fn echo(params: EchoParams) -> HandlerResult<String> {
    Ok(format!("You sent: {}", params.message))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build the router
    let router = Router::builder()
        .append_dyn("echo", echo.into_dyn()) // Register handler
        .build();

    // Create a JSON-RPC request value
    let request_value = json!({
        "jsonrpc": "2.0",
        "id": 1, // RpcId compatible ID (Number)
        "method": "echo",
        "params": { "message": "Hello RPC!" }
    });

    // Parse the request (validates JSON-RPC structure and ID)
    let rpc_request: RpcRequest = request_value.try_into()?;

    // Execute the call (no specific resources needed for this call)
    // The base router call uses its internal empty resources.
    let call_result: CallResult = router.call(rpc_request).await;

    // Process the result
    match call_result {
        Ok(response) => {
            // Success! response has { id: RpcId, method, value }
            println!("Success Response:");
            println!("  ID: {} (type: {:?})", response.id, response.id); // Display RpcId
            println!("  Method: {}", response.method);
            println!("  Value: {}", response.value); // Value is already serialized JSON
            // You would typically serialize this into a JSON-RPC response
            // let final_response = json!({"jsonrpc": "2.0", "id": response.id.to_value(), "result": response.value});
            // println!("  Final JSON: {}", final_response);
        }
        Err(call_error) => {
            // Error! call_error has { id: RpcId, method, error: rpc_router::Error }
            eprintln!("Error Response:");
            eprintln!("  ID: {} (type: {:?})", call_error.id, call_error.id); // Display RpcId
            eprintln!("  Method: {}", call_error.method);
            eprintln!("  Error: {:?}", call_error.error);
            // You would typically serialize this into a JSON-RPC error response
            // E.g., json!({"jsonrpc": "2.0", "id": call_error.id.to_value(), "error": { "code": ..., "message": ... }})
        }
    }

    Ok(())
}
```

### 2. Using Resources and Derives

This example demonstrates using shared resources and the derive macros for cleaner code.

```rust
// file: examples/02-resources-derives.rs
use rpc_router::{
    FromResources, IntoParams, RpcRequest, Resources, Router, RpcParams, RpcResource, HandlerResult, RpcId, CallResult // Added RpcId, CallResult
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

// --- Resources ---
// Use #[derive(RpcResource)] for resources handlers need.
// Must also derive Clone.
#[derive(Clone, RpcResource)]
struct AppConfig {
    greeting: String,
}

#[derive(Clone, RpcResource)]
struct Counter(Arc<tokio::sync::Mutex<u32>>); // Use Arc/Mutex for shared mutability

// --- Params ---
// Use #[derive(RpcParams)] for parameter types.
// Must also derive Deserialize.
#[derive(Deserialize, RpcParams)]
struct GreetParams {
    name: String,
}

// --- Handler ---
// Request resources and params using their types.
// Note: HandlerResult<T> implies Result<T, rpc_router::HandlerError>
async fn greet(
    config: AppConfig,      // Request AppConfig resource
    counter: Counter,       // Request Counter resource
    params: GreetParams,    // Request GreetParams
) -> HandlerResult<String> { // Return simple String or HandlerError
    let mut count = counter.0.lock().await;
    *count += 1;
    let message = format!("{}, {}! (Call #{})", config.greeting, params.name, *count);
    Ok(message)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create resources instances
    let config = AppConfig { greeting: "Hello".to_string() };
    let counter = Counter(Arc::new(tokio::sync::Mutex::new(0)));

    // Build the router and register resources
    let router = Router::builder()
        .append_dyn("greet", greet.into_dyn())
        .append_resource(config) // Add config to router's base resources
        .append_resource(counter.clone()) // Add counter
        .build();

    // --- First Call ---
    let request_value1 = json!({
        "jsonrpc": "2.0",
        "id": "req-1", // RpcId::String
        "method": "greet",
        "params": { "name": "Alice" }
    });
    let rpc_request1: RpcRequest = request_value1.try_into()?;

    // Execute using the router's base resources
    println!("--- Call 1 (ID: {}) ---", rpc_request1.id);
    match router.call(rpc_request1).await {
        Ok(res) => println!("Success: {}", res.value),
        Err(err) => eprintln!("Error: {:?}", err.error),
    }

    // --- Second Call ---
    let request_value2 = json!({
        "jsonrpc": "2.0",
        "id": 2, // RpcId::Number
        "method": "greet",
        "params": { "name": "Bob" }
    });
    let rpc_request2: RpcRequest = request_value2.try_into()?;

    println!("\n--- Call 2 (ID: {}) ---", rpc_request2.id);
    match router.call(rpc_request2).await {
        Ok(res) => println!("Success: {}", res.value),
        Err(err) => eprintln!("Error: {:?}", err.error),
    }

    // --- Check counter ---
    println!("\nFinal counter: {}", *counter.0.lock().await); // Should be 2

    Ok(())
}
```

### 3. Custom Error Handling

This demonstrates defining a custom application error and handling it.

```rust
// file: examples/03-custom-errors.rs
use rpc_router::{
    FromResources, IntoHandlerError, IntoParams, RpcRequest, Resources, Router, RpcParams,
    RpcResource, RpcHandlerError, RpcId // Note: RpcHandlerError derive, Added RpcId
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error; // Using thiserror for concise error definition

// --- Custom Application Error ---
// Use #[derive(RpcHandlerError)] to automatically implement IntoHandlerError
// It needs Debug + Send + Sync + 'static
#[derive(Debug, Error, RpcHandlerError, Serialize)] // Serialize is optional, for potential logging
pub enum TaskError {
    #[error("Task with ID {0} not found")]
    NotFound(i64),
    #[error("Task title cannot be empty")]
    TitleCannotBeEmpty,
    #[error("Database error: {0}")]
    DbError(String),
}

// --- Resource ---
#[derive(Clone, RpcResource)]
struct TaskStore {} // Placeholder for a real DB connection/client

impl TaskStore {
    // Simulate DB operations that can fail
    async fn create_task(&self, title: String) -> Result<i64, TaskError> {
        if title.is_empty() {
            Err(TaskError::TitleCannotBeEmpty)
        } else if title == "fail_db" {
            Err(TaskError::DbError("Simulated connection lost".to_string()))
        }
        else {
            Ok(12345) // Simulate successful creation with new ID
        }
    }
}

// --- Params ---
#[derive(Deserialize, RpcParams)]
struct CreateTaskParams {
    title: String,
}

// --- Handler ---
// Returns Result<i64, TaskError> - our custom error type
// The Ok value (i64) must be Serialize.
// The Err value (TaskError) must impl IntoHandlerError + Send + Sync + 'static.
async fn create_task(
    task_store: TaskStore,
    params: CreateTaskParams,
) -> Result<i64, TaskError> {
    task_store.create_task(params.title).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build router with resource
    let router = Router::builder()
        .append_dyn("create_task", create_task.into_dyn())
        .append_resource(TaskStore {})
        .build();

    // --- Test Cases ---
    let test_cases = vec![
        ("req-ok", json!({"title": "My new task"})), // RpcId::String
        (101, json!({"title": ""})),                 // RpcId::Number
        (Value::Null, json!({"title": "fail_db"})), // RpcId::Null (via json!(null))
    ];

    for (id_json, params) in test_cases {
        let id_value = json!(id_json); // Create Value for id
        println!("\n--- Testing Request ID: {} ---", id_value);
        let request_value = json!({
            "jsonrpc": "2.0",
            "id": id_value, // Use the Value here
            "method": "create_task",
            "params": params
        });
        // Parsing the RpcRequest validates id type among other things
        let rpc_request: RpcRequest = match request_value.try_into() {
             Ok(req) => req,
             Err(parse_err) => {
                eprintln!("Failed to parse request: {:?}", parse_err);
                continue;
             }
        };

        match router.call(rpc_request).await {
            Ok(response) => {
                // response.id is RpcId
                println!("Success: Request ID {}, New task ID = {}", response.id, response.value);
            }
            Err(call_error) => {
                // call_error.id is RpcId
                eprintln!("Error encountered for ID {}, method '{}'", call_error.id, call_error.method);
                // Check if it's a handler error
                if let rpc_router::Error::Handler(mut handler_error) = call_error.error {
                    // Try to extract our specific TaskError
                    // remove() consumes the inner error
                    if let Some(task_error) = handler_error.remove::<TaskError>() {
                        eprintln!("  App Error Type: TaskError");
                        eprintln!("  App Error Value: {:?}", task_error);
                        // Here you can map TaskError variants to JSON-RPC error codes/messages
                        match task_error {
                            TaskError::NotFound(_) => eprintln!("  (Would map to e.g., code -32602)"),
                            TaskError::TitleCannotBeEmpty => eprintln!("  (Would map to e.g., code -32602)"),
                            TaskError::DbError(_) => eprintln!("  (Would map to e.g., code -32000)"),
                        }
                    } else {
                        // It was a HandlerError, but not our TaskError type
                        // We can still access the debug representation
                        eprintln!("  App Error Type: Unknown (within HandlerError)");
                        eprintln!("  App Error Value: {:?}", handler_error); // Uses HandlerError's Debug impl
                    }
                } else {
                    // It was a router-level error (e.g., MethodNotFound, ParamsParsing)
                    eprintln!("  Router Error Type: {:?}", call_error.error);
                }
            }
        }
    }

    Ok(())
}
```

### 4. Router Builder Macro & Call-Specific Resources

```rust
// file: examples/04-builder-macro-call-resources.rs
use rpc_router::{
    FromResources, IntoParams, RpcRequest, Resources, Router, RpcParams, RpcResource,
    RpcHandlerError, router_builder, RpcId, IntoHandlerError // Import the macro, Added RpcId, IntoHandlerError
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

// --- Errors ---
#[derive(Debug, Error, RpcHandlerError, Serialize)]
pub enum MyError {
    #[error("Invalid Context or Permission Denied")]
    PermissionDenied,
}

// --- Resources ---
#[derive(Clone, RpcResource)]
struct CommonConfig { setting: String }

// Resource specific to a call (e.g., user context)
#[derive(Clone, RpcResource)]
struct UserContext { user_id: i64, role: String }

// --- Params ---
#[derive(Deserialize, RpcParams)]
struct GetDataParams { key: String }

// --- Handlers ---
// Returns Result<Serialize, IntoHandlerError>
async fn get_common_setting(config: CommonConfig) -> Result<String, MyError> {
    Ok(config.setting.clone())
}

// This handler requires both common and call-specific resources
// Returns Result<Serialize, IntoHandlerError>
async fn get_user_data(
    config: CommonConfig,
    user_ctx: UserContext, // Request the call-specific resource
    params: GetDataParams
) -> Result<Value, MyError> {
    if user_ctx.role != "admin" && params.key == "secret" {
        // Non-admin cannot access 'secret' key
        return Err(MyError::PermissionDenied);
    }
    Ok(json!({
        "userId": user_ctx.user_id,
        "role": user_ctx.role,
        "setting": config.setting,
        "dataForKey": format!("data_for_{}", params.key)
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use the router_builder! macro
    let router = router_builder!(
        handlers: [get_common_setting, get_user_data], // List handlers
        resources: [CommonConfig { setting: "default_value".to_string() }] // List common resources
    )
    .build();

    // --- Call 1: get_common_setting (uses only router resources) ---
    println!("--- Call 1: get_common_setting ---");
    let req1: RpcRequest = json!({
        "jsonrpc": "2.0", "id": "c1", "method": "get_common_setting"
    }).try_into()?;
    match router.call(req1).await {
        Ok(res) => println!("Success (ID {}): {}", res.id, res.value),
        Err(err) => eprintln!("Error (ID {}): {:?}", err.id, err.error),
    }

    // --- Call 2: get_user_data (needs additional UserContext) ---
    println!("\n--- Call 2: get_user_data (as Admin) ---");
    let req2: RpcRequest = json!({
        "jsonrpc": "2.0", "id": "c2-admin", "method": "get_user_data", "params": {"key": "secret"}
    }).try_into()?;

    // Build additional resources for this specific call
    let admin_resources = Resources::builder()
        .append(UserContext { user_id: 101, role: "admin".to_string() })
        .build();

    // Use call_with_resources to provide the UserContext
    match router.call_with_resources(req2, admin_resources).await {
        Ok(res) => println!("Success (ID {}): {}", res.id, res.value),
        Err(err) => eprintln!("Error (ID {}): {:?}", err.id, err.error),
    }

    // --- Call 3: get_user_data (as User, requesting secret) ---
     println!("\n--- Call 3: get_user_data (as User, requesting secret) ---");
    let req3: RpcRequest = json!({
        "jsonrpc": "2.0", "id": "c3-user-secret", "method": "get_user_data", "params": {"key": "secret"}
    }).try_into()?;

    let user_resources = Resources::builder()
        .append(UserContext { user_id: 202, role: "user".to_string() })
        .build();

    match router.call_with_resources(req3, user_resources).await {
        Ok(res) => println!("Success (ID {}): {}", res.id, res.value), // Should not happen based on logic
        Err(err) => {
            eprintln!("Error (ID {}): {:?}", err.id, err.error);
             // We can extract the specific error here too if needed
            if let rpc_router::Error::Handler(mut he) = err.error {
                if let Some(my_error) = he.remove::<MyError>() {
                     eprintln!("  -> Extracted App Error: {:?}", my_error); // Expect PermissionDenied
                }
            }
        }
    }

    Ok(())
}
```

## Conclusion

`rpc-router` provides a flexible and type-safe foundation for building JSON-RPC services in Rust. By leveraging traits (`FromResources`, `IntoParams`, `IntoHandlerError`) and derive macros (`RpcResource`, `RpcParams`, `RpcHandlerError`), you can define handlers, manage dependencies, and handle errors with minimal boilerplate, keeping your focus on the application logic. The separation between router/handler errors (`rpc_router::Error`, `HandlerError`) and application errors (`AppError`), along with the type-safe `RpcId` and validated `RpcRequest`, allows for robust, structured error handling within the framework and easy access to your specific error types when needed.
