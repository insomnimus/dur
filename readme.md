# dur
Dur is a human-readable duration parser and formatter/pretty-printer.

## `no_std` Support
Dur works without std!
It does not use the heap and therefore `alloc` is not required, enabling it to work without a memory allocator.

However, you can enable the `alloc` feature for marginally better error messages and the `std` feature for the crate's `Error` type to implement `std::error::Error`.

## Examples
```rust
// StdDuration is a re-export of core::time::Duration
use dur::{Duration, StdDuration};

// Parsing
let d = "1m 42s".parse::<Duration>().unwrap();
assert_eq!(d, Duration::from_secs(60 + 42));
// Duration::to_std and Duration::from_std convert to and from std's Duration:
assert_eq!(d.to_std(), StdDuration::from_secs(60 + 42));
assert_eq!(d, Duration::from_std(StdDuration::from_secs(60 + 42)));

// Formatting
assert_eq!("1m 42s", &format!("{d}"));
// The alternate formatter `#` makes it use full units:
assert_eq!("1 minute 42 seconds", &format!("{d:#}"));

// Fractions work:
let d = "5.1230 secs".parse::<Duration>().unwrap();
assert_eq!(d, Duration::from_millis(5000 + 123));

// Without any precision formatter, at most 2 digits after the decimal point are printed:
assert_eq!("5.12s", &format!("{d}"));

// We can specify precision:
assert_eq!("5.123s", &format!("{d:.3}"));

// Trailling zeros are removed while formatting
let d = "1.2000 milliseconds".parse::<Duration>().unwrap();
assert_eq!("1.2ms", &format!("{d}"));
// The precision specifier is considered "maximum number of digits after the decimal point"
// so, trailling zeroes are still removed!
assert_eq!("1.2ms", &format!("{d:.5}"));

// Durations are normalized to human readable forms:
let hour = "3600 seconds".parse::<Duration>().unwrap();
assert_eq!("1h", &format!("{hour}"));

// IF the string contains only a single integer, no unit, it's parsed as milliseconds:
assert_eq!("500".parse::<Duration>(), Ok(Duration::from_millis(500)));

// However if there's more than one value, it's an error:
assert_eq!(
	dur::parse("1m 300"),
	Err(dur::Error::MissingUnit),
);

// Negative values aren't allowed:
assert_eq!(
	dur::parse("-50 weeks"),
	Err(dur::Error::IsNegative(dur::Decimal::new(-50, 0))),
);

// Duration implements arithmetic traits:
let mut d = Duration::from_secs(0);
d += Duration::from_millis(50);
d -= Duration::from_millis(8);
assert_eq!(d, Duration::from_millis(42));

d  /= 2_u32;
assert_eq!(d, Duration::from_millis(21));
assert_eq!(d * 2_u32, Duration::from_millis(42));

// You can add/subtract StdDuration as well:
let sd = StdDuration::from_millis(100);
assert_eq!(sd, d + StdDuration::from_millis(79));
// It's implemented both ways:
assert_eq!(d, sd - Duration::from_millis(79));

// You can add/sub Duration from a SystemTime:
#[cfg(feature = "std")]
{
	let mut now = std::time::SystemTime::now();
	now -= Duration::from_secs(2);
	now += Duration::from_secs(50);
}

// Finally, you can also compare Duration and StdDuration:
assert_eq!(
	Duration::from_nanos(30),
	StdDuration::from_nanos(30),
);
```

## Optional Features
- `alloc`: Makes error messages marginally more informative by making `Error::InvalidUnit` store the offending string.
- `std`: Makes `Error` implement `std::error::Error`. (Automatically enables the `alloc` feature.)
- `serde`: Enables [serde](https://crates.io/crates/serde) de/serialization for [Duration]. (automatically enables the `alloc` feature)
- `clap`: Enables using `Duration` directly as an `Arg` in [clap](https://crates.io/crates/clap). (automatically enables the `std` feature)

## Syntax
Dur understands durations of the form "N UNIT" or "N1 UNIT1 N' UNIT2".

Spaces between numbers and units are optional.

Numbers can be decimal: `1.2`, `.5`, `5.`.

Numbers cannot be negative.

Units are case insensitive.

Supported units:
- nanoseconds, nanosecond, nanos, ns
- microseconds, microsecond, micros, us, µs
- milliseconds, millisecond, millis, ms
- second, seconds, secs, sec, s
- minutes, minute, mins, min, m
- hours, hour, hrs, hr, h
- days, day, d
- weeks, week, w
- years, year, yrs, yr, y

One exception is with strings that contain only one non-negative integer (e.g. `"1234"`): these are parsed as milliseconds.
