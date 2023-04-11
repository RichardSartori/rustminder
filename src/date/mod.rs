use chrono::{Datelike, Utc};
use std::fmt;
use super::{Result, Error};

type Day = u32;
type Month = u32;
type Year = i32;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Recurring {
	month: Month,
	day: Day,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Fixed {
	year: Year,
	date: Recurring,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum AnyDate {
	Recurring(Recurring),
	Fixed(Fixed),
}

impl From<Recurring> for Fixed {
	fn from(value: Recurring) -> Self {
		Fixed{ year: Utc::now().year(), date: value }
	}
}

impl fmt::Display for Recurring {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:02}/{:02}", self.day, self.month)
	}
}

impl fmt::Display for Fixed {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}/{:04}", self.date, self.year)
	}
}

impl TryFrom<&str> for Recurring {
	type Error = Error;
	fn try_from(value: &str) -> Result<Self> {
		let mut iter = value.split(',');
		let Some(day) = iter.next() else {
			return Err("missing 'day' slot");
		};
		let Some(month) = iter.next() else {
			return Err("missing 'month' slot");
		};
		if iter.next().is_some() {
			return Err("extra ',' found");
		}
		let Ok(month) = month.trim().parse::<Month>() else {
			return Err("failed to parse month");
		};
		let Ok(day) = day.trim().parse::<Day>() else {
			return Err("failed to parse day");
		};
		Ok(Recurring{ month, day })
	}
}

impl TryFrom<&str> for Fixed {
	type Error = Error;
	fn try_from(value: &str) -> Result<Self> {
		let Some(first) = value.find(',') else {
			return Err("missing first separator");
		};
		let Some(second) = value[first+1..].find(',') else {
			return Err("missing second separator");
		};
		let (date_slice, remaining) = value.split_at(first+1+second);
		let date = Recurring::try_from(date_slice)?;
		let mut iter = remaining.split(',');
		let Some(_) = iter.next() else {
			return Err("split_at failure");
		};
		let Some(year) = iter.next() else {
			return Err("missing 'year' slot");
		};
		if iter.next().is_some() {
			return Err("extra ',' found");
		}
		let Ok(year) = year.trim().parse::<Year>() else {
			return Err("failed to parse year");
		};
		Ok(Fixed{ year, date })
	}
}

impl TryFrom<&str> for AnyDate {
	type Error = Error;
	fn try_from(value: &str) -> Result<Self> {
		if let Ok(recurring) = Recurring::try_from(value) {
			return Ok(AnyDate::Recurring(recurring));
		};
		if let Ok(fixed) = Fixed::try_from(value) {
			return Ok(AnyDate::Fixed(fixed));
		}
		Err("no Date format matched")
	}
}

impl Recurring {

	pub fn new(day: Day, month: Month) -> Self {
		Recurring{ month, day }
	}

	pub fn now() -> Self {
		let now = Utc::now();
		Recurring::new(now.day(), now.month())
	}
}

fn is_leap(year: Year) -> bool {
	if year % 400 == 0 { return true; }
	if year % 100 == 0 { return false; }
	if year %   4 == 0 { return true; }
	false
}

fn last_day(month: Month, year: Year) -> Day {
	match month {
		1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
		4 | 6 | 9 | 11 => 30,
		2 => if is_leap(year) { 29 } else { 28 },
		_ => 31,
	}
}

impl Fixed {

	pub fn new(day: Day, month: Month, year: Year) -> Self {
		Fixed { year, date: Recurring::new(day, month) }
	}

	pub fn now() -> Self {
		let now = Utc::now();
		Fixed::new(now.day(), now.month(), now.year())
	}

	// return then next day
	pub fn next(self) -> Self {
		let mut next = self;
		next.date.day = next.date.day.checked_add(1).unwrap();
		if next.date.day > last_day(next.date.month, next.year) {
			next.date.day = 1;
			next.date.month = next.date.month.checked_add(1).unwrap();
		}
		if next.date.month > 12 {
			next.date.day = 1;
			next.date.month = 1;
			next.year = next.year.checked_add(1).unwrap(); // might panic
		}
		next
	}

	// return the sole date that have the same day & month
	// in the range [now, now+1*year)
	// 29/02 map to 28/02 if the range does not contain it
	pub fn next_match(self) -> Self {
		let mut next = Fixed::from(self.date);
		if next < Fixed::now() {
			next.year += 1;
		}
		if (next.date == Recurring::new(29,2)) && (!is_leap(next.year)) {
			next.date = Recurring::new(28,2)
		}
		next
	}

	// return the number of years between self and target
	// useful when target = self.next_match()
	pub fn year_diff(self, target: Self) -> i32 {
		target.year - self.year
	}

	// TODO:
	// return the number of days from self to target
	pub fn to(self, target: Self) -> u32 {
		let mut current = self;
		let mut count: u32 = 0;
		while current < target {
			current = current.next();
			count = count.checked_add(1).unwrap();
		}
		count
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use std::cmp::Ordering;

	// test Recurring
	#[test]
	fn recurring_parse_ok() {
		assert_eq!(
			Recurring::try_from("1,1").unwrap(),
			Recurring::new(1,1)
		);
	}
	#[test]
	fn recurring_parse_with_spaces() {
		assert_eq!(
			Recurring::try_from("   2  , 2    ").unwrap(),
			Recurring::new(2,2)
		);
	}
	#[test]
	fn recurring_parse_missing_day() {
		assert!(
			Recurring::try_from(",3")
			.is_err()
		);
	}
	#[test]
	fn recurring_parse_missing_month() {
		assert!(
			Recurring::try_from("4,")
			.is_err()
		);
	}
	#[test]
	fn recurring_parse_invalid() {
		assert!(
			Recurring::try_from("-5,5")
			.is_err()
		);
		assert!(
			Recurring::try_from("6,-6")
			.is_err()
		);
	}
	#[test]
	fn recurring_parse_extra_data() {
		assert!(
			Recurring::try_from("7,7,7")
			.is_err()
		);
	}
	#[test]
	fn recurring_compare_less() {
		let base = Recurring::new(1, 1);
		let different_month = Recurring::new(1, 2);
		let different_day   = Recurring::new(2, 1);
		assert_eq!(
			base.cmp(&different_month),
			Ordering::Less
		);
		assert_eq!(
			base.cmp(&different_day),
			Ordering::Less
		);
	}
	#[test]
	fn recurring_compare_equal() {
		let base = Recurring::new(1, 1);
		assert_eq!(
			base.cmp(&base),
			Ordering::Equal
		)
	}
	#[test]
	fn recurring_compare_more() {
		let base = Recurring::new(1, 1);
		let different_month = Recurring::new(1, 2);
		let different_day   = Recurring::new(2, 1);
		assert_eq!(
			different_month.cmp(&base),
			Ordering::Greater
		);
		assert_eq!(
			different_day.cmp(&base),
			Ordering::Greater
		);
	}
	#[test]
	fn recurring_compare_today() {
		let today = Recurring::now();
		let today = Fixed::from(today);
		let epoch = Fixed::new(1, 1, 1970);
		assert!(epoch < today);
	}

	// test Fixed
	#[test]
	fn fixed_parse_ok() {
		assert_eq!(
			Fixed::try_from("01,01,01").unwrap(),
			Fixed::new(1,1,1)
		);
	}
	#[test]
	fn fixed_parse_with_spaces() {
		assert_eq!(
			Fixed::try_from("   2  , 2    ,   2   ").unwrap(),
			Fixed::new(2,2,2)
		);
	}
	#[test]
	fn fixed_parse_missing_day() {
		assert!(
			Fixed::try_from(",3,3")
			.is_err()
		);
	}
	#[test]
	fn fixed_parse_missing_month() {
		assert!(
			Fixed::try_from("4,,4")
			.is_err()
		);
	}
	#[test]
	fn fixed_parse_missing_year() {
		assert!(
			Fixed::try_from("5,5")
			.is_err()
		);
	}
	#[test]
	fn fixed_parse_invalid() {
		assert!(
			Fixed::try_from("-6,-6,-6")
			.is_err()
		);
	}
	#[test]
	fn fixed_parse_extra_data() {
		assert!(
			Fixed::try_from("7,7,7,7")
			.is_err()
		);
	}
	#[test]
	fn fixed_compare_less() {
		let base = Fixed::new(1, 1, 1);
		let different_year  = Fixed::new(1, 1, 2);
		let different_month = Fixed::new(1, 2, 1);
		let different_day   = Fixed::new(2, 1, 1);
		assert_eq!(
			base.cmp(&different_year),
			Ordering::Less
		);
		assert_eq!(
			base.cmp(&different_month),
			Ordering::Less
		);
		assert_eq!(
			base.cmp(&different_day),
			Ordering::Less
		);
	}
	#[test]
	fn fixed_compare_equal() {
		let base = Fixed::new(1, 1, 1);
		assert_eq!(
			base.cmp(&base),
			Ordering::Equal
		)
	}
	#[test]
	fn fixed_compare_more() {
		let base = Fixed::new(1, 1, 1);
		let different_year  = Fixed::new(1, 1, 2);
		let different_month = Fixed::new(1, 2, 1);
		let different_day   = Fixed::new(2, 1, 1);
		assert_eq!(
			different_year.cmp(&base),
			Ordering::Greater
		);
		assert_eq!(
			different_month.cmp(&base),
			Ordering::Greater
		);
		assert_eq!(
			different_day.cmp(&base),
			Ordering::Greater
		);
	}
	#[test]
	fn fixed_compare_today() {
		let today = Fixed::now();
		let epoch = Fixed::new(1, 1, 1970);
		assert!(epoch < today);
	}

	// test AnyDate
	#[test]
	fn any_parse_recurring() {
		assert_eq!(
			AnyDate::try_from("1,1").unwrap(),
			AnyDate::Recurring(Recurring::new(1,1))
		);
	}
	#[test]
	fn any_parse_fixed() {
		assert_eq!(
			AnyDate::try_from("2,2,2").unwrap(),
			AnyDate::Fixed(Fixed::new(2,2,2))
		);
	}
	#[test]
	fn any_parse_neither() {
		assert!(
			AnyDate::try_from("3,3,")
			.is_err()
		);
	}

	// test impl Fixed
	#[test]
	fn next_day() {
		assert_eq!(
			Fixed::new(1, 1, 1970).next(),
			Fixed::new(2, 1, 1970)
		);
	}
	#[test]
	fn next_month() {
		assert_eq!(
			Fixed::new(31, 1, 1970).next(),
			Fixed::new(1, 2, 1970)
		);
	}
	#[test]
	fn next_year() {
		assert_eq!(
			Fixed::new(31, 12, 1970).next(),
			Fixed::new(1, 1, 1971)
		);
	}
	#[test]
	fn next_month_february_leap() {
		assert_eq!(
			Fixed::new(28, 2, 2000).next(),
			Fixed::new(29, 2, 2000)
		);
	}
	#[test]
	fn next_month_february_not_leap() {
		assert_eq!(
			Fixed::new(28, 2, 1900).next(),
			Fixed::new(1, 3, 1900)
		);
	}
} // mod test
