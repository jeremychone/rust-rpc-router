use crate::IntoParams;
use crate::Result;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Implements `IntoParams` for any type that also implements `IntoParams`.
///
/// Note: Application code might prefer to avoid this blanket implementation
///       and opt for enabling it on a per-type basis instead. If that's the case,
///       simply remove this general implementation and provide specific
///       implementations for each type.
impl<D> IntoParams for Option<D>
where
	D: DeserializeOwned + Send,
	D: IntoParams,
{
	fn into_params(value: Option<Value>) -> Result<Self> {
		let value = value.map(|v| serde_json::from_value(v)).transpose()?;
		Ok(value)
	}
}

/// This is the IntoParams implementation for serde_json Value.
///
/// Note: As above, this might not be a capability app code might want to
///       allow for rpc_handlers, prefering to have everything strongly type.
///       In this case, just remove this impelementation
impl IntoParams for Value {}
