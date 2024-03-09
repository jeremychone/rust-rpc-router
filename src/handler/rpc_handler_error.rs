pub type RpcHandlerResult<T> = core::result::Result<T, RpcHandlerError>;

#[derive(Debug)]
pub struct RpcHandlerError {
	pub boxed: Box<dyn std::error::Error>,
}

impl IntoRpcHandlerError for RpcHandlerError {
	fn into_handler_error(self) -> RpcHandlerError {
		self
	}
}

pub trait IntoRpcHandlerError
where
	Self: std::error::Error + Sized + 'static,
{
	fn into_handler_error(self) -> RpcHandlerError {
		RpcHandlerError { boxed: Box::new(self) }
	}
}

// region:    --- Error Boilerplate

impl core::fmt::Display for RpcHandlerError {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for RpcHandlerError {}

// endregion: --- Error Boilerplate
