use crate::input::{ResultType, StreamResult};



#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
	pub x: f64,
	pub y: f64,
	pub z: f64
}

#[derive(Debug, Clone, Copy, Default)]
pub struct State {
	pub position: Vec3,
	direction: Vec3,
	// In m/s^2
	acceleration: Vec3,
	speed: f64,
	timestamp: usize
}

static mut TIMESTAMP: usize = 3000;

impl State {
	pub fn update_state(self, items: &Vec<StreamResult>) {
		
		for item in items {
			match item.payload_type {
				ResultType::Acceleration => self.update_acceleration(&item.payload_items),
				ResultType::TruePosition => self.update_true_position(&item.payload_items),
				ResultType::Direction => self.update_direction(&item.payload_items),
				ResultType::Speed => self.update_speed(&item.payload_items),
				ResultType::Position => self.update_position(&item.payload_items),
			}
		}
	}

	pub fn get_prediction(self) -> String {
		format!("{} {} {}", self.position.x, self.position.y, self.position.z)
	}

	fn update_acceleration(mut self, data: &Vec<f64>) {
		self.acceleration = Vec3 {
			x: data[0],
			y: data[1],
			z: data[2],
		}
	}

	fn update_true_position(mut self, data: &Vec<f64>) {
		self.position = Vec3 {
			x: data[0],
			y: data[1],
			z: data[2],
		}
	}

	fn update_position(mut self, data: &Vec<f64>) {

		unsafe {
			println!("{},{},{},{},{}", TIMESTAMP.div_euclid(3000) - 1, TIMESTAMP, data[0], data[1], data[2]);
			TIMESTAMP += 3000;
		}
	}

	fn update_direction(mut self, data: &Vec<f64>) {
		self.direction = Vec3 {
			x: data[0],
			y: data[1],
			z: data[2],
		}
	}

	fn update_speed(mut self, data: &Vec<f64>) {
		self.speed = data[0];
	}
}