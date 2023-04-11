use super::*;
use std::cmp::Ordering;

#[derive(Debug, PartialEq, Eq)]
enum HolidayKind {
	Recurring(date::Recurring),
	Fixed(date::Fixed),
	Span(date::Fixed, date::Fixed),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Holiday {
	desc: String,
	kind: HolidayKind,
}

impl TryFrom<&str> for Holiday {
	type Error = Error;
	fn try_from(value: &str) -> Result<Self> {
		let mut iter = value.split(';');
		let Some(desc) = iter.next() else {
			return Err("missing 'desc' slot");
		};
		let Some(begin) = iter.next() else {
			return Err("missing 'begin' slot");
		};
		let end = iter.next();
		if iter.next().is_some() {
			return Err("extra ';' found");
		};
		let desc = String::from(desc.trim());
		if let Some(end) = end {
			let begin = date::Fixed::try_from(begin)?;
			let end = date::Fixed::try_from(end)?;
			return match begin.cmp(&end) {
				Ordering::Less => Ok(Holiday{ desc, kind: HolidayKind::Span(begin, end) }),
				Ordering::Equal => Ok(Holiday{ desc, kind: HolidayKind::Fixed(begin) }),
				Ordering::Greater => Err("begin is after end"),
			};
		}
		if let Ok(begin) = date::Recurring::try_from(begin) {
			return Ok(Holiday{ desc, kind: HolidayKind::Recurring(begin) });
		}
		if let Ok(begin) = date::Fixed::try_from(begin) {
			return Ok(Holiday{ desc, kind: HolidayKind::Fixed(begin) });
		}
		Err("no Holiday format matched")
	}
}

impl IntoEvents for Holiday {
	fn into_events(self) -> Vec<Event> {
		let mut vec: Vec<Event> = Vec::new();
		match self.kind {
			HolidayKind::Recurring(recurring) => {
				let event = Event {
					kind: EventKind::Holiday,
					date: date::Fixed::from(recurring).next_match(),
					desc: self.desc,
				};
				vec.push(event);
			},
			HolidayKind::Fixed(fixed) => {
				let event = Event {
					kind: EventKind::Holiday,
					date: fixed.next_match(),
					desc: self.desc,
				};
				vec.push(event);
			},
			HolidayKind::Span(begin, end) => {
				let mut remaining = begin.to(end)+1;
				let mut current = begin;
				while current <= end {
					remaining = remaining.checked_sub(1).unwrap();
					let event = Event {
						kind: EventKind::Holiday,
						date: current,
						desc: format!("{} ({} days remaining)", self.desc, remaining),
					};
					vec.push(event);
					current = current.next();
				}
			},
		};
		vec
	}
}

#[cfg(test)]
mod test {
	use super::*;

	fn new_recurring() -> Holiday {
		Holiday{
			desc: String::from("Christmas"),
			kind: HolidayKind::Recurring(date::Recurring::new(25,12)),
		}
	}

	fn new_fixed() -> Holiday {
		Holiday{
			desc: String::from("Easter"),
			kind: HolidayKind::Fixed(date::Fixed::new(9,4,2023)),
		}
	}

	fn new_span() -> Holiday {
		Holiday{
			desc: String::from("Summer"),
			kind: HolidayKind::Span(
				date::Fixed::new(1,7,2023),
				date::Fixed::new(31,8,2023)
			)
		}
	}

	#[test]
	fn holiday_parse_recurring() {
		assert_eq!(
			Holiday::try_from("   Christmas   ;25,12").unwrap(),
			new_recurring()
		);
	}
	#[test]
	fn holiday_parse_recurring_invalid() {
		assert_ne!(
			Holiday::try_from("Christmas;25,12,2000").unwrap(),
			new_recurring()
		);
		assert_ne!(
			Holiday::try_from("Christmas;25,12,2000;25,12,2000").unwrap(),
			new_recurring()
		);
	}
	#[test]
	fn holiday_parse_fixed() {
		assert_eq!(
			Holiday::try_from("Easter;  9 ,  4    ,2023  ").unwrap(),
			new_fixed()
		);
		assert_eq!(
			Holiday::try_from("Easter;9,4,2023;9,4,2023").unwrap(),
			new_fixed()
		);
	}
	#[test]
	fn holiday_parse_fixed_invalid() {
		assert_ne!(
			Holiday::try_from("Easter;9,4").unwrap(),
			new_fixed()
		);
	}
	#[test]
	fn holiday_parse_span() {
		assert_eq!(
			Holiday::try_from("Summer;1,7,2023;  31 ,8  ,    2023").unwrap(),
			new_span()
		);
	}
	#[test]
	fn holiday_parse_span_invalid() {
		assert!(
			Holiday::try_from("Summer;1,7,2023;31,8,2023;0,0,0")
			.is_err()
		);
		assert_ne!(
			Holiday::try_from("Summer;1,7").unwrap(),
			new_span()
		);
		assert_ne!(
			Holiday::try_from("Summer;1,7,2023").unwrap(),
			new_span()
		);
	}
} // mod test
