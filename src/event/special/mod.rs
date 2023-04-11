use super::*;

#[derive(Debug, PartialEq, Eq)]
pub struct Special {
	desc: String,
	date: date::Fixed,
}

impl TryFrom<&str> for Special {
	type Error = Error;
	fn try_from(value: &str) -> Result<Self> {
		let mut iter = value.split(';');
		let Some(desc) = iter.next() else {
			return Err("missing 'desc' slot");
		};
		let Some(date) = iter.next() else {
			return Err("missing 'date' slot");
		};
		if iter.next().is_some() {
			return Err("extra ';' found");
		};
		let desc = String::from(desc.trim());
		let date = date::Fixed::try_from(date)?;
		Ok(Special{ desc, date })
	}
}

impl IntoEvents for Special {
	fn into_events(self) -> Vec<Event> {
		let event = Event {
			kind: EventKind::Special,
			date: self.date,
			desc: self.desc,
		};
		vec![event]
	}
}

#[cfg(test)]
mod parse {
	use super::*;

	#[test]
	fn full() {
		assert_eq!(
			Special::try_from("  desc   ;1,1,1").unwrap(),
			Special{ desc: String::from("desc"), date: date::Fixed::new(1,1,1) }
		);
	}
	#[test]
	fn missing_date() {
		assert!(
			Special::try_from("desc")
			.is_err()
		);
	}
	#[test]
	fn extra_slot() {
		assert!(
			Special::try_from("desc;1,1,1;1,1,1")
			.is_err()
		);
	}
} // mod test
