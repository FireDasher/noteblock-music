use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Note {
	pub time: u32,
	pub note: u8,
}

impl Note {
	pub fn new(time: u32, note: u8) -> Self {
		Self { time, note }
	}
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Layer {
	pub name: String,
	pub instrument: u8,
	pub notes: Vec<Note>,
}

impl Layer {
	pub fn new(name: String, instrument: u8) -> Self {
		Self {
			name,
			instrument,
			notes: Vec::new()
		}
	}
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Project {
	pub layers: Vec<Layer>,
}

impl Project {
	pub fn new() -> Self {
		Self{ layers: vec![Layer::new("Layer 1".to_string(), 0)] }
	}
}