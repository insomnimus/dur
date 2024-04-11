use crate::*;

/// Implements [Display] without loss of precision.
///
/// The only purpose of [ExactDisplay] is to implement [Display] losslessly; meaning, printing a value of this type with
/// `{}` and parsing it back into [Duration] will always yield the same value.<br>
/// The only way to obtain a value of this struct is via the [format_exact](Duration::format_exact) method on [Duration].
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ExactDisplay(pub(crate) u128);

impl Display for ExactDisplay {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		let d = |ns, unit| {
			to_dec(ns)
				.map_or_else(|| Decimal::from(ns / unit), |ns| ns / Decimal::from(unit))
				.normalize()
		};

		if self.0 < MICROSECOND {
			write!(f, "{}ns", self.0)
		} else if self.0 < MILLISECOND {
			write!(f, "{}us", d(self.0, MICROSECOND))
		} else if self.0 < SECOND {
			write!(f, "{}ms", d(self.0, MILLISECOND))
		} else if self.0 < MINUTE {
			write!(f, "{}s", d(self.0, SECOND))
		} else if self.0 < HOUR {
			let (mins, ns) = sub_unit(self.0, MINUTE);
			write!(f, "{mins}m")?;
			if ns != 0 {
				write!(f, " {}s", d(ns, SECOND))?;
			}
			Ok(())
		} else if self.0 < DAY {
			let (hours, ns) = sub_unit(self.0, HOUR);
			let (mins, ns) = sub_unit(ns, MINUTE);
			write!(f, "{hours}h")?;
			if mins != 0 {
				write!(f, " {mins}m")?;
			}
			if ns != 0 {
				write!(f, " {}s", d(ns, SECOND))?;
			}
			Ok(())
		} else if self.0 < YEAR {
			let (days, ns) = sub_unit(self.0, DAY);
			let (hours, ns) = sub_unit(ns, HOUR);
			let (mins, ns) = sub_unit(ns, MINUTE);
			write!(f, "{days}d")?;
			if hours != 0 {
				write!(f, " {hours}h")?;
			}
			if mins != 0 {
				write!(f, " {mins}m")?;
			}
			if ns != 0 {
				write!(f, " {}s", d(ns, SECOND))?;
			}
			Ok(())
		} else {
			let (years, ns) = sub_unit(self.0, YEAR);
			let (days, ns) = sub_unit(ns, DAY);
			let (hours, ns) = sub_unit(ns, HOUR);
			let (mins, ns) = sub_unit(ns, MINUTE);
			write!(f, "{years}yr")?;
			if days != 0 {
				write!(f, " {days}d")?;
			}
			if hours != 0 {
				write!(f, " {hours}h")?;
			}
			if mins != 0 {
				write!(f, " {mins}m")?;
			}
			if ns != 0 {
				write!(f, " {}s", d(ns, SECOND))?;
			}
			Ok(())
		}
	}
}

fn sub_unit(n: u128, unit: u128) -> (u128, u128) {
	// let times = (n / Decimal::from(unit)).floor();
	// (
	// times.try_into().unwrap(),
	// (n - (times * Decimal::from(unit)).normalize()),
	// )
	let times = n / unit;
	(times, n - (times * unit))
}

struct Dec {
	n: Decimal,
	short: &'static str,
	long: &'static str,
}

struct Int {
	n: u128,
	short: &'static str,
	long: &'static str,
}

impl Display for Dec {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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

impl Display for Int {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if f.alternate() && self.n == 1 {
			write!(f, "1 {}", self.long)
		} else if f.alternate() {
			write!(f, "{} {}s", self.n, self.long)
		} else {
			write!(f, "{}{}", self.n, self.short)
		}
	}
}

fn d(n: u128, unit: u128, short: &'static str, long: &'static str) -> Dec {
	let n = to_dec(n).map_or_else(
		|| to_dec(n / unit).expect("value was too big"),
		|n| n / Decimal::from(unit),
	);

	Dec { n, short, long }
}

fn i(n: u128, short: &'static str, long: &'static str) -> Int {
	Int { n, short, long }
}

impl Display for Duration {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		// let n = u128::try_from(self.0.ceil()).unwrap();

		if self.0 < MICROSECOND {
			i(self.0, "ns", "nanosecond").fmt(f)
		} else if self.0 < MILLISECOND {
			d(self.0, MICROSECOND, "us", "microsecond").fmt(f)
		} else if self.0 < SECOND {
			d(self.0, MILLISECOND, "ms", "millisecond").fmt(f)
		} else if self.0 < MINUTE {
			d(self.0, SECOND, "s", "second").fmt(f)
		} else if self.0 < HOUR {
			let (mins, nanos) = sub_unit(self.0, MINUTE);
			i(mins, "m", "minute").fmt(f)?;
			let (secs, _) = sub_unit(nanos, SECOND);
			if secs != 0 {
				f.write_str(" ")?;
				i(secs, "s", "second").fmt(f)?;
			}
			Ok(())
		} else if self.0 < DAY {
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
		} else if self.0 < YEAR {
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
