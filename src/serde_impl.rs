use alloc::string::ToString;
use core::fmt;

use serde::{
	de::{
		self,
		Visitor,
	},
	Deserialize,
	Serialize,
	Serializer,
};

use crate::{
	serde_impl::de::Deserializer,
	Duration,
};

impl Serialize for Duration {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let s = self.format_exact().to_string();
		serializer.serialize_str(&s)
	}
}

struct DurationVisitor;

impl<'de> Visitor<'de> for DurationVisitor {
	type Value = Duration;

	fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str("a non-negative integer or a string describing a duration")
	}

	fn visit_u8<E>(self, n: u8) -> Result<Duration, E>
	where
		E: de::Error,
	{
		Ok(Duration(n as u128))
	}

	fn visit_u16<E>(self, n: u16) -> Result<Duration, E>
	where
		E: de::Error,
	{
		Ok(Duration(n as u128))
	}

	fn visit_u32<E>(self, n: u32) -> Result<Duration, E>
	where
		E: de::Error,
	{
		Ok(Duration(n as u128))
	}

	fn visit_u64<E>(self, n: u64) -> Result<Duration, E>
	where
		E: de::Error,
	{
		Ok(Duration(n as u128))
	}

	fn visit_u128<E>(self, n: u128) -> Result<Duration, E>
	where
		E: de::Error,
	{
		Ok(Duration(n))
	}

	fn visit_str<E>(self, s: &str) -> Result<Duration, E>
	where
		E: de::Error,
	{
		crate::parse(s).map_err(|e| E::custom(e.to_string()))
	}
}

impl<'de> Deserialize<'de> for Duration {
	fn deserialize<D>(deserializer: D) -> Result<Duration, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_str(DurationVisitor)
	}
}
