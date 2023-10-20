Parse the human-readable duration string into a [Duration].

For the allowed syntax, see the [crate level documentation](crate).

#### Examples
```rust
let tests = [
	("15ms", 15),
	("1 second", 1000),
	("55", 55),
	("1m 2s", 60000 + 2000),
];

for (s, in_ms) in tests {
	let d = dur::parse(s).expect("failed to parse!");
	assert_eq!(d.as_millis(), in_ms);
}

use dur::{Decimal, Error};

let should_error = [
	("2 foo", Error::InvalidUnit("foo".into())),
	("2m 5", Error::MissingUnit),
	("     2    ", Error::InvalidDuration),
	("50000000000000000000000000000 years", Error::ValueTooBig),
	("-4.2s", Error::IsNegative(Decimal::new(-42, 1))),
];

for (s, error) in should_error {
	let res = dur::parse(s);
	assert_eq!(res, Err(error));
}
```
