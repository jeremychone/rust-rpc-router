use crate::{Error, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;

/// `IntoParams` allows for converting an `Option<Value>` into
/// the necessary type for RPC handler parameters.
/// The default implementation below will result in failure if the value is `None`.
/// For customized behavior, users can implement their own `into_params`
/// method.
pub trait IntoRpcParams: DeserializeOwned + Send {
	fn into_params(value: Option<Value>) -> Result<Self> {
		match value {
			Some(value) => Ok(serde_json::from_value(value)?),
			None => Err(Error::RpcIntoParamsMissing),
		}
	}
}

/// Marker trait with a blanket implementation that return T::default
/// if the `params: Option<Value>` is none.
pub trait IntoDefaultRpcParams: DeserializeOwned + Send + Default {}

impl<P> IntoRpcParams for P
where
	P: IntoDefaultRpcParams,
{
	fn into_params(value: Option<Value>) -> Result<Self> {
		match value {
			Some(value) => Ok(serde_json::from_value(value)?),
			None => Ok(Self::default()),
		}
	}
}

// region:    --- Blanket implementation

// IMPORTANT: Probably need to be put below a feature, like `with-blanket-option-params`

/// Implements `IntoRpcParams` for any type that also implements `IntoRpcParams`.
///
/// Note: Application code might prefer to avoid this blanket implementation.
impl<D> IntoRpcParams for Option<D>
where
	D: DeserializeOwned + Send,
	D: IntoRpcParams,
{
	fn into_params(value: Option<Value>) -> Result<Self> {
		let value = value.map(|v| serde_json::from_value(v)).transpose()?;
		Ok(value)
	}
}

// IMPORTANT: Probably need to be put below a feature, like `with-blanket-value-params`

/// This is the IntoRpcParams implementation for serde_json Value.
///
/// Note: As above, this might not be a capability app code might want to
///       allow for rpc_handlers, prefering to have everything strongly type.
impl IntoRpcParams for Value {}

// endregion: --- Blanket implementation
