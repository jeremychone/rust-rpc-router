use crate::RpcResources;

/// Macro generatring the RpcHandler implementations for zero or more FromRpcResources with the last argument being IntoRpcParams
/// and one with not last IntoRpcParams argument.
#[macro_export]
macro_rules! impl_rpc_handler_pair {
    ($K:ty, $($T:ident),*) => {

		// RpcHandler implementations for zero or more FromRpcResources with the last argument being IntoRpcParams
        impl<F, Fut, $($T,)* P, R, E> $crate::RpcHandler<($($T,)*), (P,), R> for F
        where
            F: FnOnce($($T,)* P) -> Fut + Clone + Send + 'static,
            $( $T: $crate::FromRpcResources+ Clone + Send + Sync + 'static, )*
            P: $crate::IntoRpcParams + Send + Sync + 'static,
            R: serde::Serialize + Send + Sync + 'static,
            E: $crate::IntoRpcHandlerError,
            Fut: futures::Future<Output = core::result::Result<R, E>> + Send,
        {
            type Future = $crate::handler::PinFutureValue;

			#[allow(unused)] // somehow resources will be marked as unused
            fn call(
                self,
                resources: RpcResources,
                params_value: Option<serde_json::Value>,
            ) -> Self::Future {
                Box::pin(async move {
                    let param = P::into_params(params_value)?;

                    let res = self(
                        $( $T::from_resources(&resources)?, )*
                        param,
                    ).await;

                    match res {
                        Ok(result) => Ok(serde_json::to_value(result)?),
                        Err(ex) => {
                            let he = $crate::IntoRpcHandlerError::into_handler_error(ex);
                            Err(he.into())
                        },
                    }
                })
            }
        }

       // RpcHandler implementations for zero or more FromRpcResources and NO IntoRpcParams
       impl<F, Fut, $($T,)* R, E> $crate::RpcHandler<($($T,)*), (), R> for F
       where
               F: FnOnce($($T,)*) -> Fut + Clone + Send + 'static,
               $( $T: $crate::FromRpcResources + Clone + Send + Sync + 'static, )*
               R: serde::Serialize + Send + Sync + 'static,
               E: $crate::IntoRpcHandlerError,
               Fut: futures::Future<Output = core::result::Result<R, E>> + Send,
       {
               type Future = $crate::handler::PinFutureValue;

               #[allow(unused)] // somehow resources will be marked as unused
               fn call(
                       self,
                       resources: RpcResources,
                       _params: Option<serde_json::Value>,
               ) -> Self::Future {
                       Box::pin(async move {
                            let res = self(
                                    $( $T::from_resources(&resources)?, )*
                            ).await;

                            match res {
                                Ok(result) => Ok(serde_json::to_value(result)?),
                                Err(ex) => {
                                    let he = $crate::IntoRpcHandlerError::into_handler_error(ex);
                                    Err(he.into())
                                },
                            }

                       })
               }
       }
    };

}

impl_rpc_handler_pair!(RpcResources,);
impl_rpc_handler_pair!(RpcResources, T1);
impl_rpc_handler_pair!(RpcResources, T1, T2);
impl_rpc_handler_pair!(RpcResources, T1, T2, T3);
impl_rpc_handler_pair!(RpcResources, T1, T2, T3, T4);
impl_rpc_handler_pair!(RpcResources, T1, T2, T3, T4, T5);
