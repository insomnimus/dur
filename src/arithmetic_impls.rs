use core::{
	ops::{
		Add,
		AddAssign,
		Div,
		DivAssign,
		Mul,
		MulAssign,
		Rem,
		RemAssign,
		Sub,
		SubAssign,
	},
	time::Duration as StdDuration,
};
#[cfg(feature = "std")]
use std::time::SystemTime;

use crate::Duration;

// Addition

impl Add<Duration> for Duration {
	type Output = Self;

	fn add(self, rhs: Self) -> Self {
		Self(self.0 + rhs.0)
	}
}

impl Add<StdDuration> for Duration {
	type Output = Self;

	fn add(self, rhs: StdDuration) -> Self {
		Self(self.0 + rhs.as_nanos())
	}
}

#[cfg(feature = "std")]
impl Add<Duration> for SystemTime {
	type Output = Self;

	fn add(self, rhs: Duration) -> Self {
		self + rhs.to_std()
	}
}

impl Add<Duration> for StdDuration {
	type Output = Self;

	fn add(self, rhs: Duration) -> Self {
		self + rhs.to_std()
	}
}
impl AddAssign<Duration> for Duration {
	fn add_assign(&mut self, rhs: Self) {
		self.0 += rhs.0;
	}
}

impl AddAssign<StdDuration> for Duration {
	fn add_assign(&mut self, rhs: StdDuration) {
		self.0 += rhs.as_nanos();
	}
}

impl AddAssign<Duration> for StdDuration {
	fn add_assign(&mut self, rhs: Duration) {
		*self += rhs.to_std();
	}
}

#[cfg(feature = "std")]
impl AddAssign<Duration> for SystemTime {
	fn add_assign(&mut self, rhs: Duration) {
		*self += rhs.to_std();
	}
}

// Subtraction

impl Sub<Duration> for Duration {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self {
		Self(self.0 - rhs.0)
	}
}

impl Sub<StdDuration> for Duration {
	type Output = Self;

	fn sub(self, rhs: StdDuration) -> Self {
		Self(self.0 - rhs.as_nanos())
	}
}

#[cfg(feature = "std")]
impl Sub<Duration> for SystemTime {
	type Output = Self;

	fn sub(self, rhs: Duration) -> Self {
		self - rhs.to_std()
	}
}

impl Sub<Duration> for StdDuration {
	type Output = Self;

	fn sub(self, rhs: Duration) -> Self {
		self - rhs.to_std()
	}
}

impl SubAssign<Duration> for Duration {
	fn sub_assign(&mut self, rhs: Self) {
		self.0 -= rhs.0;
	}
}

impl SubAssign<StdDuration> for Duration {
	fn sub_assign(&mut self, rhs: StdDuration) {
		self.0 -= rhs.as_nanos();
	}
}

impl SubAssign<Duration> for StdDuration {
	fn sub_assign(&mut self, rhs: Duration) {
		*self -= rhs.to_std();
	}
}

#[cfg(feature = "std")]
impl SubAssign<Duration> for SystemTime {
	fn sub_assign(&mut self, rhs: Duration) {
		*self -= rhs.to_std();
	}
}

// Multiplication
impl<T: Into<u128>> Mul<T> for Duration {
	type Output = Self;

	fn mul(self, rhs: T) -> Self {
		Self(self.0 * rhs.into())
	}
}

impl<T: Into<u128>> MulAssign<T> for Duration {
	fn mul_assign(&mut self, rhs: T) {
		self.0 *= rhs.into();
	}
}

// Division
impl<T: Into<u128>> Div<T> for Duration {
	type Output = Self;

	fn div(self, rhs: T) -> Self {
		Self(self.0 / rhs.into())
	}
}

impl<T: Into<u128>> DivAssign<T> for Duration {
	fn div_assign(&mut self, rhs: T) {
		self.0 /= rhs.into();
	}
}

// Modulus

impl<T: Into<u128>> Rem<T> for Duration {
	type Output = Self;

	fn rem(self, rhs: T) -> Self {
		Self(self.0 % rhs.into())
	}
}

impl<T: Into<u128>> RemAssign<T> for Duration {
	fn rem_assign(&mut self, rhs: T) {
		self.0 %= rhs.into();
	}
}
