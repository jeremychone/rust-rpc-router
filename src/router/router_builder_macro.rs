/// A simple macro to create a new RouterBuider from a list of handlers
/// and optionaly a list of resources
///
/// ## Pattern 1 - List of function handlers
/// ```
/// router_builder!(
///   create_project,
///   list_projects,
///   update_project,
///   delete_project
/// );
/// ```
/// Is equivalent to:
/// ```
/// RouterBuilder::default()
///     .append_dyn("create_project", create_project.into_box())
///     .append_dyn("list_projects", list_projects.into_box())
///     .append_dyn("update_project", update_project.into_box())
///     .append_dyn("delete_project", delete_project.into_box())
/// ```
///
/// ## Pattern 2 - List of function handlers, and resources
/// ```
/// router_builder!(
///   handlers: [get_task, create_task],         // will be turned into routes
///   resources: [ModelManager {}, AiManager {}] // common resources for all calls
/// );
/// ```
///
/// Is equivalent to:
///
/// ```
/// RouterBuilder::default()
///     .append_dyn("get_task", get_task.into_box())
///     .append_dyn("create_task", create_task.into_box())
///     .append_resource(ModelManager {})
///     .append_resource(AiManager {})
/// ```
///
/// ## Pattern 3 - Just for consistency with Pattern 2, we can have omit the resources
///
/// ```
/// router_builder!(
///   handlers: [get_task, create_task]
/// );
/// ```
///
#[macro_export]
macro_rules! router_builder {
	// Pattern 1 - with `rpc_router!(my_fn1, myfn2)`
    ($($fn_name:ident),+ $(,)?) => {
        {
					use rpc_router::{Handler, RouterBuilder};

					let mut builder = RouterBuilder::default();
					$(
							builder = builder.append_dyn(stringify!($fn_name), $fn_name.into_dyn());
					)+
					builder
        }
    };

    // Pattern 2 - `rpc_router!(handlers: [my_fn1, myfn2], resources: [ModelManger {}, AiManager {}])`
    (handlers: [$($handler:ident),* $(,)?], resources: [$($resource:expr),* $(,)?]) => {{
        use rpc_router::{Handler, RouterBuilder};

        let mut builder = RouterBuilder::default();
        $(
            builder = builder.append_dyn(stringify!($handler), $handler.into_dyn());
        )*
        $(
            builder = builder.append_resource($resource);
        )*
        builder
    }};

    // Pattern 3 - with `rpc_router!(handlers: [my_fn1, myfn2])`
    (handlers: [$($handler:ident),* $(,)?]) => {{
        use rpc_router::{Handler, RouterBuilder};

        let mut builder = RouterBuilder::default();
        $(
            builder = builder.append_dyn(stringify!($handler), $handler.into_dyn());
        )*
        builder
    }};
}
