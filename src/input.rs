use std::{f64, net::UdpSocket};

use anyhow::Ok;

#[derive(Debug, Clone, Copy)]
pub enum ResultType {
	TruePosition,
	Speed,
	Acceleration,
	Direction,
}

#[derive(Debug, Clone)]
pub struct StreamResult {
	pub payload_type: ResultType,
	pub payload_items: Vec<f64>,
}

pub fn create_connection(port: u16) -> anyhow::Result<UdpSocket> {
	let url = format!("localhost:{}", port);

	let socket = UdpSocket::bind(&url)?;

	return Ok(socket);
}

pub fn write_data(socket: &UdpSocket, data: &[u8]) -> anyhow::Result<()> {

	let _res = socket.send_to(data, "localhost:4242")?;

	// println!("BYTES SENT: {}", res);


	return Ok(());
}

pub fn initialize_stream(socket: &UdpSocket) -> anyhow::Result<()> {
	let mut buf: [u8; 1024] = [0;1024];

	write_data(socket, "READY".as_bytes())?;

	loop {
		let (_res, _source) = socket.recv_from(&mut buf)?;
		
		let received = String::from_utf8(buf.to_vec())?;

		println!("init: recieved: {}", received);

		if received.starts_with("Trajectory Generated!") {
			break;
		}

		buf = [0;1024];
	}

	Ok(())
}

pub fn read_input(socket: &UdpSocket) -> anyhow::Result<Vec<StreamResult>> {
	let mut buf: [u8; 1024] = [0;1024];
	let mut results: Vec<StreamResult> = vec![];
	let mut idx = 0;

	loop {
		let (res, _source) = socket.recv_from(&mut buf)?;

		let mut resized_buf = buf.to_vec();
		resized_buf.resize(res, 0);


		let received = String::from_utf8(resized_buf)?;
	
		let reading: Vec<&str> = received.split('\n').collect();

		dbg!(reading[0]);

		if idx == 0 && reading[0] != "MSG_START" {
			panic!("No MSG_START_COMMAND");
		} else if reading[0] == "MSG_START" {
			idx += 1;
			continue;
		} else if reading[0] == "MSG_END" {
			break;
		}

		let (timestamp, command) = reading[0].split_at(14);

		println!("TS: {}, CMD: {}", timestamp, command);

		let result_type = match &command {
			&"ACCELERATION" => ResultType::Acceleration,
			&"DIRECTION" => ResultType::Direction,
			&"SPEED" => ResultType::Speed,
			&"TRUE POSITION" => ResultType::TruePosition,
			_ => panic!("Unknown RESULT_TYPE")
		};

		let items: Vec<f64> = reading.into_iter()
			.skip(1)
			.filter_map(|f| f.parse::<f64>().ok())
			.collect();

		dbg!(&items);

		results.push(StreamResult { payload_type: result_type, payload_items: items });

		idx += 1;
	}

	return Ok(results);
}
