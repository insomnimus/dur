#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../readme.md")]

#[cfg(feature = "alloc")]
extern crate alloc;

mod arithmetic_impls;
#[cfg(feature = "clap")]
mod clap_arg;
mod formatting;
#[cfg(feature = "serde")]
mod serde_impl;
#[cfg(test)]
mod tests;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::boxed::Box;
#[doc(no_inline)]
pub use core::time::Duration as StdDuration;
use core::{
	fmt::{
		self,
		Debug,
		Display,
		Formatter,
	},
	str::FromStr,
};

pub use formatting::ExactDisplay;
use nom::{
	branch::alt,
	bytes::complete::{
		tag,
		tag_no_case,
	},
	character::complete::{
		digit1,
		one_of,
		space0,
	},
	combinator::{
		opt,
		recognize,
		success,
		value,
	},
	sequence::{
		pair,
		separated_pair,
	},
};
#[doc(no_inline)]
pub use rust_decimal::{
	self,
	Decimal,
};

/// A human readable duration backed by a [u128].
///
/// The underlying [u128] represents the duration in nanoseconds.
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash, Default)]
pub struct Duration(u128);

const MICROSECOND: u128 = 1000;
const MILLISECOND: u128 = MICROSECOND * 1000;
const SECOND: u128 = MILLISECOND * 1000;
const MINUTE: u128 = SECOND * 60;
const HOUR: u128 = MINUTE * 60;
const DAY: u128 = HOUR * 24;
const WEEK: u128 = DAY * 7;
const YEAR: u128 = SECOND * 31_557_600;

// Error

/// The parse error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
	/// Catch-all for values that aren't proper durations.
	InvalidDuration,
	/// The value being parsed is too big in nanoseconds in total to fit in a
	/// [u128] or bigger than [Decimal::MAX] in case of a single unit.
	ValueTooBig,
	/// The value being parsed is missing a unit.
	///
	/// Note that values without any unit and only one number, such as `"42"`
	/// are not errors and are parsed as milliseconds.
	MissingUnit,
	/// The value being parsed contains negative durations.
	IsNegative(Decimal),
	/// The value contains an unrecognized duration unit.
	#[cfg(feature = "alloc")]
	InvalidUnit(Box<str>),
	#[cfg(not(feature = "alloc"))]
	/// The value contains an unrecognized duration unit.
	InvalidUnit,
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Self::InvalidDuration => write!(f, "invalid duration"),
			Self::ValueTooBig => write!(f, "the duration value is too big to store"),
			Self::MissingUnit => write!(f, "missing unit after number"),
			Self::IsNegative(d) => write!(f, "durations cannot be negative ({d})"),
			#[cfg(feature = "alloc")]
			Self::InvalidUnit(s) => write!(f, "invalid duration unit `{s}`"),
			#[cfg(not(feature = "alloc"))]
			Self::InvalidUnit => write!(f, "invalid duration unit`"),
		}
	}
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

// Parsing

fn to_dec(n: u128) -> Option<Decimal> {
	// Decimal::try_from and Decimal::from both panic with values greater than
	// Decimal::MAX as below
	if n > 79228162514264337593543950335 {
		None
	} else {
		Some(Decimal::from(n))
	}
}

fn parse_unit(input: &str) -> Result<(&str, u128), Error> {
	if input.trim().is_empty() {
		return Err(Error::MissingUnit);
	}

	let (rem, unit) = alt((
		value(
			1,
			alt((
				tag_no_case("nanoseconds"),
				tag_no_case("nanosecond"),
				tag_no_case("nanos"),
				tag_no_case("ns"),
			)),
		),
		value(
			MICROSECOND,
			alt((
				tag_no_case("microseconds"),
				tag_no_case("microsecond"),
				tag_no_case("micros"),
				tag_no_case("us"),
				tag_no_case("µs"),
			)),
		),
		value(
			MILLISECOND,
			alt((
				tag_no_case("milliseconds"),
				tag_no_case("millisecond"),
				tag_no_case("millis"),
				tag_no_case("ms"),
			)),
		),
		value(
			SECOND,
			alt((
				tag_no_case("seconds"),
				tag_no_case("second"),
				tag_no_case("secs"),
				tag_no_case("sec"),
				tag_no_case("s"),
			)),
		),
		value(
			MINUTE,
			alt((
				tag_no_case("minutes"),
				tag_no_case("minute"),
				tag_no_case("mins"),
				tag_no_case("min"),
				tag_no_case("m"),
			)),
		),
		value(
			HOUR,
			alt((
				tag_no_case("hours"),
				tag_no_case("hour"),
				tag_no_case("hrs"),
				tag_no_case("hr"),
				tag_no_case("h"),
			)),
		),
		value(
			DAY,
			alt((tag_no_case("days"), tag_no_case("day"), tag_no_case("d"))),
		),
		value(
			WEEK,
			alt((tag_no_case("weeks"), tag_no_case("week"), tag_no_case("w"))),
		),
		value(
			YEAR,
			alt((
				tag_no_case("years"),
				tag_no_case("year"),
				tag_no_case("yrs"),
				tag_no_case("yr"),
				tag_no_case("y"),
			)),
		),
	))(input)
	.map_err(|_: nom::Err<nom::error::Error<_>>| {
		#[cfg(not(feature = "alloc"))]
		return Error::InvalidUnit;
		#[cfg(feature = "alloc")]
		Error::InvalidUnit(
			input
				.split_whitespace()
				.next()
				.unwrap_or_else(|| input.trim())
				.into(),
		)
	})?;

	if rem.starts_with(|c: char| c.is_alphabetic()) {
		#[cfg(feature = "alloc")]
		return Err(Error::InvalidUnit(
			input.split_whitespace().next().unwrap_or(input).into(),
		));

		#[cfg(not(feature = "alloc"))]
		Err(Error::InvalidUnit)
	} else {
		Ok((rem, unit))
	}
}

#[doc = include_str!("fn.parse.md")]
pub fn parse(input: &str) -> Result<Duration, Error> {
	if input.trim().is_empty() {
		return Err(Error::InvalidDuration);
	}
	if let Ok(d) = input.parse::<Decimal>() {
		if d.is_sign_negative() {
			return Err(Error::IsNegative(d));
		}
		return d
			.checked_mul(Decimal::from(MILLISECOND))
			.map(|d| Duration(u128::try_from(d).unwrap()))
			.ok_or(Error::ValueTooBig);
	}

	let parse_decimal = alt((
		recognize(separated_pair(digit1, tag("."), digit1)),
		recognize(pair(digit1, tag("."))),
		recognize(pair(tag("."), digit1)),
		digit1,
	));

	let mut parse_decimal = recognize(pair(opt(one_of("-+")), parse_decimal));

	let mut sep = alt::<_, _, nom::error::Error<_>, _>((
		recognize(pair(tag(","), space0)),
		space0,
		success(""),
	));

	let mut s = input;
	let mut n = 0_u128;

	for i in 0.. {
		if i != 0 {
			(s, _) = sep(s).unwrap();
		}

		if s.is_empty() {
			break;
		}

		let (rem, d) =
			parse_decimal(s).map_err(|_: nom::Err<nom::error::Error<_>>| Error::InvalidDuration)?;

		let d = d.parse::<Decimal>().map_err(|e| match e {
			rust_decimal::Error::ExceedsMaximumPossibleValue
			| rust_decimal::Error::LessThanMinimumPossibleValue => Error::ValueTooBig,
			_ => Error::InvalidDuration,
		})?;

		if d.is_sign_negative() {
			return Err(Error::IsNegative(d));
		}

		let rem = rem.trim_start_matches(|c: char| c == ' ' || c == '\t');
		let (rem, unit) = parse_unit(rem)?;
		let d = Decimal::from(unit)
			.checked_mul(d)
			.ok_or(Error::ValueTooBig)?;
		n = n
			.checked_add(d.try_into().unwrap())
			.ok_or(Error::ValueTooBig)?;
		s = rem;
	}

	Ok(Duration(n))
}

/// Parse the human-readable duration string into an [StdDuration].
///
/// See [parse] for usage.
pub fn parse_std(input: &str) -> Result<StdDuration, Error> {
	parse(input).map(|d| d.to_std())
}

/// Constructs a new [Duration]. Equivalent to [Duration::from]
pub fn pretty(d: StdDuration) -> Duration {
	Duration::from(d)
}

// Conversions

impl FromStr for Duration {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		parse(s)
	}
}

impl From<StdDuration> for Duration {
	fn from(d: StdDuration) -> Self {
		Self::from_std(d)
	}
}

impl From<Duration> for StdDuration {
	fn from(d: Duration) -> Self {
		d.to_std()
	}
}

// Constants
impl Duration {
	pub const HOUR: Self = Self(HOUR);
	pub const MICROSECOND: Self = Self(MICROSECOND);
	pub const MILLISECOND: Self = Self(MILLISECOND);
	pub const MINUTE: Self = Self(MINUTE);
	pub const NANOSECOND: Self = Self(1);
	pub const SECOND: Self = Self(SECOND);
}

// Impls

impl Duration {
	/// Creates a new `Duration` from the specified number of nanoseconds.
	pub const fn from_nanos(ns: u128) -> Self {
		Self(ns)
	}

	/// Creates a new `Duration` from the specified number of microseconds.
	///
	/// #### Overflow Behavior
	/// IF the value in nanoseconds overflows a [u128], the behavior is the same
	/// as with [u128] overflow with multiplication.
	pub const fn from_micros(us: u128) -> Self {
		Self(us * MICROSECOND)
	}

	/// Creates a new `Duration` from the specified number of milliseconds.
	///
	/// #### Overflow Behavior
	/// IF the value in nanoseconds overflows a [u128], the behavior is the same
	/// as with [u128] overflow with multiplication.
	pub const fn from_millis(ms: u128) -> Self {
		Self(ms * MILLISECOND)
	}

	/// Creates a new `Duration` from the specified number of seconds.
	///
	/// #### Overflow Behavior
	/// IF the value in nanoseconds overflows a [u128], the behavior is the same
	/// as with [u128] overflow with multiplication.
	pub const fn from_secs(secs: u128) -> Self {
		Self(secs * SECOND)
	}

	/// Convert to [StdDuration]. equivalent to calling [Into::into].
	///
	/// #### Panics
	/// Panics if `self` is too big for an [StdDuration].
	pub fn to_std(self) -> StdDuration {
		self.try_to_std()
			.expect("the value is too big to converty to std::time::Duration")
	}

	/// Tries to convert `self` into an [StdDuration].
	///
	/// Returns [None] if the value is too big for [StdDuration].
	pub fn try_to_std(self) -> Option<StdDuration> {
		u64::try_from(self.0)
			.map(StdDuration::from_nanos)
			.or_else(|_| u64::try_from(self.as_millis()).map(StdDuration::from_millis))
			.or_else(|_| u64::try_from(self.as_secs()).map(StdDuration::from_secs))
			.ok()
	}

	/// Convert from [StdDuration]. Equivalent to [Duration::from].
	pub const fn from_std(d: StdDuration) -> Self {
		Self(d.as_nanos())
	}

	/// Returns the total number of nanoseconds contained by this Duration.
	pub const fn as_nanos(self) -> u128 {
		self.0
	}

	/// Returns the total number of whole microseconds contained by this
	/// Duration.
	pub const fn as_micros(self) -> u128 {
		self.0 / MICROSECOND
	}

	/// Returns this duration in microseconds as a [Decimal].
	pub fn as_micros_dec(self) -> Decimal {
		to_dec(self.0).map_or_else(
			|| Decimal::from(self.as_micros()),
			|n| n / Decimal::ONE_THOUSAND,
		)
	}

	/// Returns the total number of whole milliseconds contained by this
	/// Duration.
	pub const fn as_millis(self) -> u128 {
		self.0 / MILLISECOND
	}

	/// Returns this duration in milliseconds as a [Decimal].
	pub fn as_millis_dec(self) -> Decimal {
		to_dec(self.0).map_or_else(
			|| Decimal::from(self.as_millis()),
			|d| d / Decimal::from(MILLISECOND),
		)
	}

	/// Returns the total number of whole seconds contained by this Duration.
	pub const fn as_secs(self) -> u128 {
		self.0 / SECOND
	}

	/// Returns this duration in seconds as a [Decimal].
	pub fn as_secs_dec(self) -> Decimal {
		to_dec(self.0).map_or_else(
			|| Decimal::from(self.as_secs()),
			|d| d / Decimal::from(SECOND),
		)
	}

	/// Returns true if this Duration is 0.
	pub const fn is_zero(self) -> bool {
		self.0 == 0
	}

	/// Returns a struct with a lossless [Display] implementation.
	pub fn format_exact(self) -> ExactDisplay {
		ExactDisplay(self.0)
	}
}

// Trait impls
impl PartialEq<Duration> for StdDuration {
	fn eq(&self, rhs: &Duration) -> bool {
		self.as_nanos() == rhs.as_nanos()
	}
}

impl PartialEq<StdDuration> for Duration {
	fn eq(&self, rhs: &StdDuration) -> bool {
		self.as_nanos() == rhs.as_nanos()
	}
}
