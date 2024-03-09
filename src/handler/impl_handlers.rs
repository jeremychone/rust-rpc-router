use crate::Resources;

/// Macro generatring the Rpc Handler implementations for zero or more FromResources with the last argument being IntoParams
/// and one with not last IntoParams argument.
#[macro_export]
macro_rules! impl_handler_pair {
    ($K:ty, $($T:ident),*) => {

		// Handler implementations for zero or more FromResources with the last argument being IntoParams
        impl<F, Fut, $($T,)* P, R, E> $crate::Handler<($($T,)*), (P,), R> for F
        where
            F: FnOnce($($T,)* P) -> Fut + Clone + Send + 'static,
            $( $T: $crate::FromResources+ Clone + Send + Sync + 'static, )*
            P: $crate::IntoParams + Send + Sync + 'static,
            R: serde::Serialize + Send + Sync + 'static,
            E: $crate::IntoHandlerError,
            Fut: futures::Future<Output = core::result::Result<R, E>> + Send,
        {
            type Future = $crate::handler::PinFutureValue;

			#[allow(unused)] // somehow resources will be marked as unused
            fn call(
                self,
                resources: Resources,
                params_value: Option<serde_json::Value>,
            ) -> Self::Future {
                Box::pin(async move {
                    let param = P::into_params(params_value)?;

                    let res = self(
                        $( $T::from_resources(&resources)?, )*
                        param,
                    ).await;

                    match res {
                        Ok(result) => Ok(serde_json::to_value(result).map_err($crate::Error::HandlerResultSerialize)?),
                        Err(ex) => {
                            let he = $crate::IntoHandlerError::into_handler_error(ex);
                            Err(he.into())
                        },
                    }
                })
            }
        }

       // Handler implementations for zero or more FromResources and NO IntoParams
       impl<F, Fut, $($T,)* R, E> $crate::Handler<($($T,)*), (), R> for F
       where
               F: FnOnce($($T,)*) -> Fut + Clone + Send + 'static,
               $( $T: $crate::FromResources + Clone + Send + Sync + 'static, )*
               R: serde::Serialize + Send + Sync + 'static,
               E: $crate::IntoHandlerError,
               Fut: futures::Future<Output = core::result::Result<R, E>> + Send,
       {
               type Future = $crate::handler::PinFutureValue;

               #[allow(unused)] // somehow resources will be marked as unused
               fn call(
                       self,
                       resources: Resources,
                       _params: Option<serde_json::Value>,
               ) -> Self::Future {
                       Box::pin(async move {
                            let res = self(
                                    $( $T::from_resources(&resources)?, )*
                            ).await;

                            match res {
                                Ok(result) => Ok(serde_json::to_value(result).map_err($crate::Error::HandlerResultSerialize)?),
                                Err(ex) => {
                                    let he = $crate::IntoHandlerError::into_handler_error(ex);
                                    Err(he.into())
                                },
                            }

                       })
               }
       }
    };

}

impl_handler_pair!(Resources,);
impl_handler_pair!(Resources, T1);
impl_handler_pair!(Resources, T1, T2);
impl_handler_pair!(Resources, T1, T2, T3);
impl_handler_pair!(Resources, T1, T2, T3, T4);
impl_handler_pair!(Resources, T1, T2, T3, T4, T5);
impl_handler_pair!(Resources, T1, T2, T3, T4, T5, T6);
impl_handler_pair!(Resources, T1, T2, T3, T4, T5, T6, T7);
impl_handler_pair!(Resources, T1, T2, T3, T4, T5, T6, T7, T8);
