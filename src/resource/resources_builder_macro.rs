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
macro_rules! resources_builder {
    ($($x:expr),*) => {
        {
            let mut temp = rpc_router::ResourcesBuilder::default();
            $(
                temp = temp.append($x);
            )*
            temp
        }
    };
}
