#[macro_export]
macro_rules! impl_all_rpc_handlers {
	() => {
		rpc_router::impl_rpc_handler_pair!(RpcResources,);
		rpc_router::impl_rpc_handler_pair!(RpcResources, T1);
		rpc_router::impl_rpc_handler_pair!(RpcResources, T1, T2);
		rpc_router::impl_rpc_handler_pair!(RpcResources, T1, T2, T3);
		rpc_router::impl_rpc_handler_pair!(RpcResources, T1, T2, T3, T4);
		rpc_router::impl_rpc_handler_pair!(RpcResources, T1, T2, T3, T4, T5);
	};
}

/// Macro generatring the RpcHandler implementations for zero or more FromResources with the last argument being IntoParams
/// and one with not last IntoParams argument.
#[macro_export]
macro_rules! impl_rpc_handler_pair {
    ($K:ty, $($T:ident),*) => {


				// RpcHandler implementations for zero or more FromResources with the last argument being IntoParams
        impl<F, Fut, $($T,)* P, R> rpc_router::RpcHandler<$K, ($($T,)*), (P,), R> for F
        where
            F: FnOnce($($T,)* P) -> Fut + Clone + Send + 'static,
            $( $T: rpc_router::FromResources<$K> + Send + Sync + 'static, )*
            P: rpc_router::IntoParams + Send + Sync + 'static,
            R: serde::Serialize + Send + Sync + 'static,
            Fut: futures::Future<Output = $crate::Result<R>> + Send,
        {
            type Future = rpc_router::PinFutureValue;

						#[allow(unused)] // somehow resources will be marked as unused
            fn call(
                self,
                resources: $K,
                params_value: Option<serde_json::Value>,
            ) -> Self::Future {
                Box::pin(async move {
                    let param = P::into_params(params_value)?;

                    let result = self(
                        $( $T::from_resources(&resources)?, )*
                        param,
                    )
                    .await?;
                    Ok(serde_json::to_value(result)?)
                })
            }
        }

				// RpcHandler implementations for zero or more FromResources and NO IntoParams
				impl<F, Fut, $($T,)* R> rpc_router::RpcHandler<$K,($($T,)*), (), R> for F
				where
						F: FnOnce($($T,)*) -> Fut + Clone + Send + 'static,
						$( $T: rpc_router::FromResources<$K> + Send + Sync + 'static, )*
						R: serde::Serialize + Send + Sync + 'static,
						Fut: futures::Future<Output = $crate::Result<R>> + Send,
				{
						type Future = rpc_router::PinFutureValue;

						#[allow(unused)] // somehow resources will be marked as unused
						fn call(
								self,
								resources: $K,
								_params: Option<serde_json::Value>,
						) -> Self::Future {
								Box::pin(async move {
										let result = self(
												$( $T::from_resources(&resources)?, )*
										)
										.await?;
										Ok(serde_json::to_value(result)?)
								})
						}
				}
    };

}
