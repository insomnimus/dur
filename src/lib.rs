#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::boxed::Box;
use core::{
	fmt,
	str::FromStr,
	time::Duration as StdDuration,
};

use nom::{
	branch::alt,
	bytes::complete::{
		tag,
		tag_no_case,
	},
	character::complete::{
		digit1,
		space0,
	},
	combinator::{
		map_res,
		recognize,
		success,
		value,
	},
	error::{
		ErrorKind,
		FromExternalError,
		ParseError,
	},
	sequence::{
		pair,
		separated_pair,
		tuple,
	},
	IResult,
};
use rust_decimal::Decimal;

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub struct Duration(Decimal);

const MICROSECOND: u64 = 1000;
const MILLISECOND: u64 = MICROSECOND * 1000;
const SECOND: u64 = MILLISECOND * 1000;
const MINUTE: u64 = SECOND * 60;
const HOUR: u64 = MINUTE * 60;
const DAY: u64 = HOUR * 24;
const WEEK: u64 = DAY * 7;
const YEAR: u64 = SECOND * 31_557_600;

// Error

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
	InvalidDuration,
	ValueTooBig,
	InvalidUnit(Box<str>),
	MissingUnit,
}

impl<I> FromExternalError<I, Error> for Error {
	fn from_external_error(_: I, _: ErrorKind, e: Error) -> Self {
		e
	}
}

impl<I> ParseError<I> for Error {
	fn from_error_kind(_: I, _: ErrorKind) -> Self {
		Self::InvalidDuration
	}

	fn append(_: I, _: ErrorKind, other: Self) -> Self {
		other
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::InvalidDuration => write!(f, "invalid duration"),
			Self::ValueTooBig => write!(f, "the duration value is too big to store"),
			Self::MissingUnit => write!(f, "missing unit after number"),
			Self::InvalidUnit(s) => write!(f, "invalid duration unit `{s}`"),
		}
	}
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

// Parsing

fn parse_unit(input: &str) -> IResult<&str, u64, Error> {
	if input.trim().is_empty() {
		return Err(nom::Err::Failure(Error::MissingUnit));
	}

	let res: IResult<_, _, nom::error::Error<_>> = alt((
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
	))(input);

	res.map_err(|_| {
		nom::Err::Error(Error::InvalidUnit(
			input
				.split_whitespace()
				.next()
				.unwrap_or_else(|| input.trim())
				.into(),
		))
	})
}

pub fn parse(input: &str) -> Result<Duration, Error> {
	if input.trim().is_empty() {
		return Err(Error::InvalidDuration);
	}
	if let Ok(d) = input.parse::<Decimal>() {
		return d
			.checked_mul(Decimal::from(MILLISECOND))
			.map(Duration)
			.ok_or(Error::ValueTooBig);
	}

	let parse_decimal = alt((
		recognize(separated_pair(digit1, tag("."), digit1)),
		recognize(pair(digit1, tag("."))),
		recognize(pair(tag("."), digit1)),
		digit1,
	));

	let parse_decimal = map_res(parse_decimal, |s: &str| {
		s.parse::<Decimal>().map_err(|_| Error::ValueTooBig)
	});

	let mut sep = alt::<_, _, nom::error::Error<_>, _>((
		recognize(pair(tag(","), space0)),
		space0,
		success(""),
	));

	let mut parse_duration = map_res(
		tuple((parse_decimal, space0, parse_unit)),
		|(n, _, unit)| Decimal::from(unit).checked_mul(n).ok_or(Error::ValueTooBig),
	);

	let mut s = input;
	let mut n = Decimal::ZERO;
	for i in 0.. {
		if i != 0 {
			(s, _) = sep(s).unwrap();
		}
		if s.is_empty() {
			break;
		}
		let (rem, d) = parse_duration(s).map_err(|e| match e {
			nom::Err::Error(e) | nom::Err::Failure(e) => e,
			_ => unreachable!(),
		})?;
		s = rem;
		n = n.checked_add(d).ok_or(Error::ValueTooBig)?;
	}

	Ok(Duration(n))
}

pub fn parse_std(input: &str) -> Result<StdDuration, Error> {
	parse(input).map(|d| d.to_std())
}

pub fn pretty(d: StdDuration) -> Duration {
	Duration::from(d)
}

// Formatting

fn sub_unit(n: Decimal, unit: u64) -> (u64, Decimal) {
	let times = (n / Decimal::from(unit)).floor();
	(
		times.normalize().try_into().unwrap(),
		(n - (times * Decimal::from(unit)).normalize()),
	)
}

struct Dec {
	n: Decimal,
	short: &'static str,
	long: &'static str,
}

struct Int {
	n: u64,
	short: &'static str,
	long: &'static str,
}

impl fmt::Display for Dec {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if f.alternate() && self.n == Decimal::ONE {
			write!(f, "1 {}", self.long)
		} else if f.alternate() {
			write!(
				f,
				"{} {}s",
				self.n
					.trunc_with_scale(f.precision().unwrap_or(2_usize) as u32)
					.normalize(),
				self.long
			)
		} else {
			write!(
				f,
				"{}{}",
				self.n
					.trunc_with_scale(f.precision().unwrap_or(2_usize) as u32)
					.normalize(),
				self.short
			)
		}
	}
}

impl fmt::Display for Int {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if f.alternate() && self.n == 1 {
			write!(f, "1 {}", self.long)
		} else if f.alternate() {
			write!(f, "{} {}s", self.n, self.long)
		} else {
			write!(f, "{}{}", self.n, self.short)
		}
	}
}

fn d(n: Decimal, short: &'static str, long: &'static str) -> Dec {
	Dec { n, short, long }
}

fn i(n: u64, short: &'static str, long: &'static str) -> Int {
	Int { n, short, long }
}

impl fmt::Display for Duration {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let n = u64::try_from(self.0.ceil()).unwrap();

		if self.0 < Decimal::ONE_THOUSAND {
			d(self.0, "ns", "nanosecond").fmt(f)
		} else if n < MILLISECOND {
			d(self.0 / Decimal::ONE_THOUSAND, "us", "microsecond").fmt(f)
		} else if n < SECOND {
			d(self.0 / Decimal::from(MILLISECOND), "ms", "millisecond").fmt(f)
		} else if n < MINUTE {
			d(self.0 / Decimal::from(SECOND), "s", "second").fmt(f)
		} else if n < HOUR {
			let (mins, nanos) = sub_unit(self.0, MINUTE);
			i(mins, "m", "minute").fmt(f)?;
			let (secs, _) = sub_unit(nanos, SECOND);
			if secs != 0 {
				f.write_str(" ")?;
				i(secs, "s", "second").fmt(f)?;
			}
			Ok(())
		} else if n < DAY {
			let (hours, nanos) = sub_unit(self.0, HOUR);
			let (mins, nanos) = sub_unit(nanos, MINUTE);
			let (secs, _) = sub_unit(nanos, SECOND);
			i(hours, "h", "hour").fmt(f)?;
			if mins != 0 {
				f.write_str(" ")?;
				i(mins, "m", "minute").fmt(f)?;
			}
			if secs != 0 {
				f.write_str(" ")?;
				i(secs, "s", "second").fmt(f)?;
			}
			Ok(())
		} else if n < YEAR {
			let (days, nanos) = sub_unit(self.0, DAY);
			let (hours, nanos) = sub_unit(nanos, HOUR);
			let (mins, _) = sub_unit(nanos, MINUTE);
			i(days, "d", "day").fmt(f)?;
			if hours != 0 {
				f.write_str(" ")?;
				i(hours, "h", "hour").fmt(f)?;
			}
			if mins != 0 {
				f.write_str(" ")?;
				i(mins, "m", "minute").fmt(f)?;
			}
			Ok(())
		} else {
			let (years, nanos) = sub_unit(self.0, YEAR);
			let (days, nanos) = sub_unit(nanos, DAY);
			let (hours, _) = sub_unit(nanos, HOUR);
			i(years, "yr", "year").fmt(f)?;
			if days != 0 {
				f.write_str(" ")?;
				i(days, "d", "day").fmt(f)?;
			}
			if hours != 0 {
				f.write_str(" ")?;
				i(hours, "h", "hour").fmt(f)?;
			}
			Ok(())
		}
	}
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
		Self(d.as_nanos().into())
	}
}

impl From<Duration> for StdDuration {
	fn from(d: Duration) -> Self {
		Self::from_nanos(d.0.try_into().unwrap_or(u64::MAX))
	}
}

// Impls
impl Duration {
	pub fn to_std(&self) -> StdDuration {
		(*self).into()
	}

	pub fn from_std(d: StdDuration) -> Self {
		d.into()
	}

	pub fn as_nanos(&self) -> Decimal {
		// self.0.try_into().unwrap()
		self.0
	}

	pub fn as_micros(&self) -> Decimal {
		self.0 / Decimal::from(MICROSECOND)
	}

	pub fn as_millis(&self) -> Decimal {
		self.0 / Decimal::from(MILLISECOND)
	}

	pub fn as_secs(&self) -> Decimal {
		self.0 / Decimal::from(SECOND)
	}
}
