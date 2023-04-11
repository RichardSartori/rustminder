use super::*;
use colored::*;
use std::{fmt, cmp};

pub mod person;
pub mod holiday;
pub mod special;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum EventKind {
	Birthday,
	SaintDay,
	Wedding,
	Holiday,
	Special,
}

impl fmt::Display for EventKind {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let text = match self {
			EventKind::Birthday => "birthday".red(),
			EventKind::SaintDay => "saint day".blue(),
			EventKind::Wedding => "wedding anniversary".green(),
			EventKind::Holiday => "holiday".yellow(),
			EventKind::Special => "special".cyan(),
		};
		write!(f, "{}", text)
	}
}

pub const KIND_LIST: [EventKind; 5] = [
	EventKind::Birthday,
	EventKind::SaintDay,
	EventKind::Wedding,
	EventKind::Holiday,
	EventKind::Special,
];

pub struct Event {
	pub kind: EventKind,
	pub date: date::Fixed,
	pub desc: String,
}

pub trait IntoEvents {
	fn into_events(self) -> Vec<Event>;
}

fn extract(line: &str) -> Result<Vec<Event>> {
	let mut iter = line.split('=');
	let Some(event_kind) = iter.next() else {
		return Err("missing 'event kind' slot");
	};
	let Some(event) = iter.next() else {
		return Err("missing 'event' slot");
	};
	if iter.next().is_some() {
		return Err("extra '=' found");
	};
	if event_kind.trim() == "person" {
		return Ok(person::Person::try_from(event)?.into_events());
	}
	if event_kind.trim() == "holiday" {
		return Ok(holiday::Holiday::try_from(event)?.into_events());
	}
	if event_kind.trim() == "special" {
		return Ok(special::Special::try_from(event)?.into_events());
	}
	Err("no EventKind matched")
}

// parse line and add events to vector
pub fn add_from(line: &str, vec: &mut Vec<Event>) -> Result<()> {
	let events = extract(line)?;
	for event in events {
		vec.push(event);
	}
	Ok(())
}

// return vector of references to next events of kind
pub fn get_next(events: &Vec<Event>, kind: EventKind) -> Vec<&Event> {
	let now = date::Fixed::now();
	let mut next: Vec<&Event> = Vec::new();
	let filtered = events.iter().filter(|e| (e.kind == kind) && (e.date >= now));
	for event in filtered {
		match next.get(0) {
			None => next.push(event),
			Some(e) => match event.date.cmp(&e.date) {
				cmp::Ordering::Less => { next.clear(); next.push(event); },
				cmp::Ordering::Equal => next.push(event),
				cmp::Ordering::Greater => (),
			},
		}
	}
	next
}
