use std::{
	fs::{self, File, ReadDir},
	io::{BufRead, BufReader, Lines},
	path::{Path, PathBuf},
};
use super::*;

// return the default location of .rce files
pub fn default_location() -> PathBuf {
	const ROOT: &str = env!("CARGO_MANIFEST_DIR");
	const LOCATION: &str = "data/";
	let mut retval = PathBuf::new();
	retval.push(ROOT);
	retval.push(LOCATION);
	retval
}

pub struct RceIterator {
	data: ReadDir,
}

// return an iterator over the .rce files in <path>
pub fn find_rce<P: AsRef<Path>>(path: P) -> Result<RceIterator> {
	let data = match fs::read_dir(path) {
		Ok(data) => data,
		Err(_) => { return Err("could not read data folder"); },
	};
	Ok(RceIterator { data })
}

impl Iterator for RceIterator {
	type Item = PathBuf;
	fn next(&mut self) -> Option<Self::Item> {
		loop {
			let Some(Ok(entry)) = self.data.next() else {
				return None;
			};
			let Ok(filetype) = entry.file_type() else {
				continue;
			};
			if !filetype.is_file() {
				continue;
			}
			let path = entry.path();
			let Some(ext) = path.extension() else {
				continue;
			};
			if ext != "rce" {
				continue;
			}
			break Some(entry.path());
		}
	}
}

pub struct SanitizedLinesIterator {
	data: Lines<BufReader<File>>,
}

// return an iterator over the non empty lines of <filename>
pub fn read_lines<P: AsRef<Path>>(filename: P) -> Result<SanitizedLinesIterator> {
	let file = match File::open(filename) {
		Ok(file) => file,
		Err(_) => { return Err("could not read file"); },
	};
	let data = BufReader::new(file).lines();
	Ok(SanitizedLinesIterator{ data })
}

impl Iterator for SanitizedLinesIterator {
	type Item = String;
	fn next(&mut self) -> Option<Self::Item> {
		loop {
			let Some(Ok(line)) = self.data.next() else {
				return None;
			};
			let mut iter = line.as_str().split('#');
			let Some(sanitized_line) = iter.next() else {
				//panic!("split returned empty iterator");
				continue;
			};
			if sanitized_line.is_empty() {
				continue;
			}
			break Some(String::from(sanitized_line))
		}
	}
}
