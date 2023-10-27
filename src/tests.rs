#![cfg(not(feature = "alloc"))]
compile_error!("you need to enable the alloc feature to run tests");

use alloc::{
	format,
	string::ToString,
};

use crate::*;

#[test]
fn to_from_std() {
	for ms in (0..1 << 20).map(|n| n * 100) {
		let s = StdDuration::from_millis(ms);
		let d = Duration::from_std(s);
		assert_eq!(s, d.to_std());
		assert_eq!(d, Duration::from_std(d.to_std()));
	}
}

#[test]
fn parse() {
	// let us = |n| n * MICROSECOND;
	let ms = |n| n * MILLISECOND;
	let s = |n| n * SECOND;
	let m = |n| n * MINUTE;
	let h = |n| n * HOUR;
	let d = |n| n * DAY;
	let w = |n| n * WEEK;
	let y = |n| n * YEAR;

	let tests = [
		("5", ms(5)),
		("5m 2 seconds", m(5) + s(2)),
		("52 day 2nanoseconds", d(52) + 2),
		("1h1m1s", h(1) + m(1) + s(1)),
		("0.1 s 0.1 sec 0.1secs", ms(300)),
		("2\tweek", w(2)),
		("5. yrs .5h", y(5) + m(30)),
	];

	for (text, val) in tests {
		assert_eq!(text.parse::<Duration>(), Ok(Duration(val)));
	}
}

#[test]
fn as_conversions() {
	let d = Duration(MINUTE);
	let s = d.to_std();

	let tests = [
		(d.as_secs(), s.as_secs() as u128, 60),
		(d.as_millis(), s.as_millis(), 60 * 1000),
		(d.as_micros(), s.as_micros(), 60 * 1_000_000),
		(d.as_nanos(), s.as_nanos(), 60 * 1_000_000_000),
	];

	for (d_res, s_res, val) in tests {
		assert_eq!(d_res, s_res);
		assert_eq!(val, d_res);
	}
}

#[test]
fn parse_format() {
	let tests = [
		("1ns", "1 nanosecond"),
		("1.24us", "1.24 microseconds"),
		("4.2ms", "4.2 milliseconds"),
		("1.55s", "1.55 seconds"),
		("6m 3s", "6 minutes 3 seconds"),
		("40m", "40 minutes"),
		("1h 1m 1s", "1 hour 1 minute 1 second"),
		("1h 1s", "1 hour 1 second"),
		("1m 5s", "1 minute 5 seconds"),
		("5h 5m", "5 hours 5 minutes"),
		("1d", "1 day"),
		("3d 2h", "3 days 2 hours"),
		("1d 2h 3m", "1 day 2 hours 3 minutes"),
		("10yr", "10 years"),
		("10yr 5d", "10 years 5 days"),
		("4yr 3d 5h", "4 years 3 days 5 hours"),
		("10yr 5h", "10 years 5 hours"),
	];

	for (short, long) in tests {
		let a = short.parse::<Duration>().unwrap();
		let b = long.parse::<Duration>().unwrap();
		assert_eq!(
			a, b,
			"\nparsing {short:?} and {long:?} yielded different results"
		);
		assert_eq!(short, &a.to_string());
		assert_eq!(long, &format!("{b:#}"));

		let exact = a.format_exact().to_string();
		assert_eq!(short, &exact);
	}
}

#[cfg_attr(feature = "serde", test)]
#[cfg(feature = "serde")]
fn serde() {
	let tests = [
		("42", "42ms"),
		("1.5m 4.2ms", "1m 30.0042s"),
		("0.5d, 40m, 30.123450s", "12h 40m 30.12345s"),
		("90009ms", "1m 30.009s"),
	];

	for (sa, sb) in tests {
		let a = sa.parse::<Duration>().unwrap();
		let b = sb.parse::<Duration>().unwrap();
		assert_eq!(a, b, "\nnot equal: {sa:?}, {sb:?}");
		let ser = serde_json::to_string(&a).unwrap();
		assert_eq!(format!("{sb:?}"), ser, "\nserialized form doesn't match");
	}
}
