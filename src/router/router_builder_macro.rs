/// A simple macro to create a new RpcRouterInner
/// and add each rpc handler-compatible function along with their corresponding names.
///
/// e.g.,
///
/// ```
/// rpc_router!(
///   create_project,
///   list_projects,
///   update_project,
///   delete_project
/// );
/// ```
/// Is equivalent to:
/// ```
/// RpcRouterBuilder::default()
///     .append_dyn("create_project", create_project.into_box())
///     .append_dyn("list_projects", list_projects.into_box())
///     .append_dyn("update_project", update_project.into_box())
///     .append_dyn("delete_project", delete_project.into_box())
/// ```
#[macro_export]
macro_rules! router_builder {
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
}
