use super::*;

#[derive(Debug, PartialEq, Eq)]
pub struct Person {
	name: String,
	birthday: Option<date::AnyDate>,
	saint_day: Option<date::Recurring>,
	wedding_day: Option<date::AnyDate>,
}

fn parse_name(value: &str) -> Result<String> {
	let mut iter = value.split(',');
	let Some(first_name) = iter.next() else {
		return Err("missing 'first_name' slot");
	};
	let Some(last_name) = iter.next() else {
		return Err("missing 'last_name' slot");
	};
	let Some(nickname) = iter.next() else {
		return Err("missing 'nickname' slot");
	};
	if iter.next().is_some() {
		return Err("extra ',' found");
	}
	let first_name = first_name.trim();
	let last_name = last_name.trim();
	let nickname = nickname.trim();
	if !nickname.is_empty() {
		return Ok(format!("{}", nickname));
	}
	if first_name.is_empty() {
		return Err("at least first_name or nickname must be provided");
	}
	if last_name.is_empty() {
		return Ok(format!("{}", first_name));
	}
	Ok(format!("{} {}", first_name, last_name))
}

impl TryFrom<&str> for Person {
	type Error = Error;
	fn try_from(value: &str) -> Result<Self> {
		let mut iter = value.split(';');
		let Some(name) = iter.next() else {
			return Err("missing 'name' slot");
		};
		let Some(birthday) = iter.next() else {
			return Err("missing 'birthday' slot");
		};
		let Some(saint_day) = iter.next() else {
			return Err("missing 'saint_day' slot");
		};
		let Some(wedding_day) = iter.next() else {
			return Err("missing 'wedding_day' slot");
		};
		if iter.next().is_some() {
			return Err("extra ';' found");
		}
		let name = parse_name(name)?;
		let birthday = if birthday.trim().is_empty() {
			None
		} else {
			Some(date::AnyDate::try_from(birthday)?)
		};
		let saint_day = if saint_day.trim().is_empty() {
			None
		} else {
			Some(date::Recurring::try_from(saint_day)?)
		};
		let wedding_day = if wedding_day.trim().is_empty() {
			None
		} else {
			Some(date::AnyDate::try_from(wedding_day)?)
		};
		Ok(Person{name, birthday, saint_day, wedding_day})
	}
}

fn get_next_and_diff(date: date::AnyDate) -> (date::Fixed, Option<i32>) {
	match date {
		date::AnyDate::Recurring(recurring) => {
			let fixed = date::Fixed::from(recurring);
			let next = fixed.next_match();
			(next, None)
		},
		date::AnyDate::Fixed(fixed) => {
			let next = fixed.next_match();
			let diff = fixed.year_diff(next);
			(next, Some(diff))
		},
	}
}

impl IntoEvents for Person {
	fn into_events(self) -> Vec<Event> {
		let mut vec: Vec<Event> = Vec::new();
		if let Some(birthday) = self.birthday {
			let (date, age) = get_next_and_diff(birthday);
			let desc = match age {
				None => format!("{}", self.name),
				Some(age) => format!("{} (age {})", self.name, age),
			};
			let event = Event {
				kind: EventKind::Birthday,
				date: date,
				desc: desc,
			};
			vec.push(event);
		};
		if let Some(saint_day) = self.saint_day {
			let event = Event {
				kind: EventKind::SaintDay,
				date: date::Fixed::from(saint_day).next_match(),
				desc: self.name.clone(),
			};
			vec.push(event);
		};
		if let Some(wedding_day) = self.wedding_day {
			let (date, year) = get_next_and_diff(wedding_day);
			let desc = match year {
				None => format!("{}", self.name),
				Some(year) => format!("{} (year {})", self.name, year),
			};
			let event = Event {
				kind: EventKind::Wedding,
				date: date,
				desc: desc,
			};
			vec.push(event);
		};
		vec
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn name_parse_full() {
		assert_eq!(
			parse_name("Richard, SARTORI, me").unwrap(),
			String::from("me")
		);
	}
	#[test]
	fn name_parse_nickname_only() {
		assert_eq!(
			parse_name(",,Rick").unwrap(),
			String::from("Rick")
		);
	}
	#[test]
	fn name_parse_with_spaces() {
		assert_eq!(
			parse_name(",,  with spaces  ").unwrap(),
			String::from("with spaces")
		);
	}
	#[test]
	fn name_parse_first_name_only() {
		assert_eq!(
			parse_name("Richard,,").unwrap(),
			String::from("Richard")
		);
	}
	#[test]
	fn name_parse_both_names() {
		assert_eq!(
			parse_name("Richard,SARTORI,").unwrap(),
			String::from("Richard SARTORI")
		);
	}
	#[test]
	fn name_parse_last_name_only() {
		assert!(
			parse_name(",SARTORI,")
			.is_err()
		);
	}
	#[test]
	fn name_parse_missing_slot() {
		assert!(
			parse_name("Richard,SARTORI")
			.is_err()
		);
	}
	#[test]
	fn name_parse_extra_slot() {
		assert!(
			parse_name("Richard,SARTORI,Rick,me")
			.is_err()
		);
	}

	fn new_person(
		name: &str,
		birthday: Option<date::AnyDate>,
		saint_day: Option<date::Recurring>,
		wedding_day: Option<date::AnyDate>
	) -> Person {
		Person{
			name: String::from(name),
			birthday: birthday,
			saint_day: saint_day,
			wedding_day: wedding_day,
		}
	}

	#[test]
	fn person_parse_full() {
		assert_eq!(
			Person::try_from("a,b,c;1,1,1;2,2;3,3,3").unwrap(),
			new_person(
				"c",
				Some(date::AnyDate::Fixed(date::Fixed::new(1,1,1))),
				Some(date::Recurring::new(2,2)),
				Some(date::AnyDate::Fixed(date::Fixed::new(3,3,3)))
			)
		);
	}
	#[test]
	fn person_parse_missing_nickname() {
		assert_eq!(
			Person::try_from("a,b,;1,1,1;2,2;3,3,3").unwrap(),
			new_person(
				"a b",
				Some(date::AnyDate::Fixed(date::Fixed::new(1,1,1))),
				Some(date::Recurring::new(2,2)),
				Some(date::AnyDate::Fixed(date::Fixed::new(3,3,3)))
			)
		);
	}
	#[test]
	fn person_parse_with_spaces() {
		assert_eq!(
			Person::try_from(" a , b , c ; 1 , 1 , 1 ; 2 , 2 ; 3 , 3 , 3 ").unwrap(),
			new_person(
				"c",
				Some(date::AnyDate::Fixed(date::Fixed::new(1,1,1))),
				Some(date::Recurring::new(2,2)),
				Some(date::AnyDate::Fixed(date::Fixed::new(3,3,3)))
			)
		);
	}
	#[test]
	fn person_parse_without_birthday() {
		assert_eq!(
			Person::try_from("a,b,c;;2,2;3,3,3").unwrap(),
			new_person(
				"c",
				None,
				Some(date::Recurring::new(2,2)),
				Some(date::AnyDate::Fixed(date::Fixed::new(3,3,3)))
			)
		);
	}
	#[test]
	fn person_parse_without_birthday_year() {
		assert_eq!(
			Person::try_from("a,b,c;1,1;2,2;3,3,3").unwrap(),
			new_person(
				"c",
				Some(date::AnyDate::Recurring(date::Recurring::new(1,1))),
				Some(date::Recurring::new(2,2)),
				Some(date::AnyDate::Fixed(date::Fixed::new(3,3,3)))
			)
		);
	}
	#[test]
	fn person_parse_without_saint() {
		assert_eq!(
			Person::try_from("a,b,c;1,1,1;;3,3,3").unwrap(),
			new_person(
				"c",
				Some(date::AnyDate::Fixed(date::Fixed::new(1,1,1))),
				None,
				Some(date::AnyDate::Fixed(date::Fixed::new(3,3,3)))
			)
		);
	}
	#[test]
	fn person_parse_without_wedding() {
		assert_eq!(
			Person::try_from("a,b,c;1,1,1;2,2;").unwrap(),
			new_person(
				"c",
				Some(date::AnyDate::Fixed(date::Fixed::new(1,1,1))),
				Some(date::Recurring::new(2,2)),
				None
			)
		);
	}
	#[test]
	fn person_parse_without_wedding_year() {
		assert_eq!(
			Person::try_from("a,b,c;1,1,1;2,2;3,3").unwrap(),
			new_person(
				"c",
				Some(date::AnyDate::Fixed(date::Fixed::new(1,1,1))),
				Some(date::Recurring::new(2,2)),
				Some(date::AnyDate::Recurring(date::Recurring::new(3,3)))
			)
		);
	}
	#[test]
	fn person_parse_empty() {
		assert_eq!(
			Person::try_from("a,b,c;;;").unwrap(),
			new_person("c",None,None,None)
		);
	}
	#[test]
	fn person_parse_missing_slot() {
		assert!(
			Person::try_from("a,b,c;1,1,1;2,2")
			.is_err()
		);
	}
	#[test]
	fn person_parse_extra_slot() {
		assert!(
			Person::try_from("a,b,c;1,1,1;2,2;3,3,3;4,4,4")
			.is_err()
		);
	}
	#[test]
	fn person_parse_invalid_saint() {
		assert!(
			Person::try_from("a,b,c;1,1,1;2,2,2;3,3,3")
			.is_err()
		);
	}
} // mod test
