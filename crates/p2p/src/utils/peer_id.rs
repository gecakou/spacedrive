use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct PeerId(pub(crate) libp2p::PeerId);

impl FromStr for PeerId {
	type Err = libp2p::core::ParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self(libp2p::PeerId::from_str(s)?))
	}
}

impl Display for PeerId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

// TODO: Replace this with transparent when the new Specta release is merged
// TODO: 	#[cfg_attr(feature = "specta", derive(specta::Type))]
// TODO: 	pub struct PeerId(#[cfg_attr(feature = "specta", specta(type = String))] pub(crate) libp2p::PeerId);
#[cfg(feature = "specta")]
impl specta::Type for PeerId {
	const NAME: &'static str = "PeerId";
	const SID: specta::TypeSid = specta::sid!();
	const IMPL_LOCATION: specta::ImplLocation = specta::impl_location!();

	fn inline(opts: specta::DefOpts, generics: &[specta::DataType]) -> specta::DataType {
		<String as specta::Type>::inline(opts, generics)
	}

	fn reference(opts: specta::DefOpts, generics: &[specta::DataType]) -> specta::DataType {
		<String as specta::Type>::reference(opts, generics)
	}

	fn definition(opts: specta::DefOpts) -> specta::DataTypeExt {
		<String as specta::Type>::definition(opts)
	}
}
