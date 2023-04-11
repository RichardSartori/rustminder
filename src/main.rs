use calendar::{date, event, file, Result};

fn main() -> Result<()> {

	let mut events: Vec<event::Event> = Vec::new();

	let location = file::default_location();
	for path in file::find_rce(location)? {
		println!("found file \"{}\"", path.display());
		for line in file::read_lines(path)? {
			event::add_from(&line, &mut events)?;
		}
	}

	for kind in event::KIND_LIST {
		let filter = event::get_next(&events, kind);
		let mut iter = filter.iter();
		let mut msg: String;
		let now = date::Fixed::now();
		match iter.next() {
			None => msg = String::from("none found"),
			Some(e) => {
				let date = e.date;
				if date == now {
					msg = String::from("Today!");
				} else {
					msg = format!("{} (in {} days)", date, now.to(date));
				}
				msg += format!(": {}", e.desc).as_str();
				for e in iter {
					msg += format!(", {}", e.desc).as_str();
				}
			},
		}
		println!("next {}: {}", kind, msg);
	}

	Ok(())
}
