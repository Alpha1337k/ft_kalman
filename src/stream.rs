use std::{io::{BufRead, BufReader, Read, Stderr}, process::{exit, Child, Command, Stdio}, sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError}, thread::{self, JoinHandle}, time::Duration};
use std::result::Result::Ok;

use anyhow::{Error};
use duct::{cmd, Handle, ReaderHandle};

use crate::solver::Vec3;

pub fn get_stream_output(rx: &Receiver<Result<Vec3, String>>) -> anyhow::Result<(bool, Vec<Vec3>)> {
	let mut is_alive = true;
	let mut results = vec![];

	loop {
		let result = match rx.recv_timeout(Duration::from_millis(50)) {
			Ok(Ok(lines)) => lines,
			Ok(Err(lines)) => {
				is_alive = false;

				return Err(Error::msg(lines));
			},
			Err(RecvTimeoutError::Timeout) => break,
			Err(RecvTimeoutError::Disconnected) => {
				is_alive = false;

				break;
			},
		};

		results.push(result);

		break;
	}


	return anyhow::Ok((is_alive, results));
}

fn parse_stdout(line: &str) -> Option<Vec3> {
	let res = Vec3::default();

	if line.starts_with("x :") == false {
		return None;
	}

	let values: Vec<f64> = line
		.split(", ")
		.map(|l| (l[4..]).parse::<f64>().unwrap())
		.collect();

	assert!(values.len() == 3);

	Some(Vec3 {
		x: -values[0],
		y: -values[1],
		z: values[2],
	})
}

pub fn run_stream(path: &str) -> (Receiver<Result<Vec3, String>>, Sender<String>) {
	let (tx, rx) = mpsc::channel::<Result<Vec3, String>>();
	let (kill_tx, kill_rx) = mpsc::channel::<String>();

	let copy = path.to_string();

	let thd = thread::spawn(move || {
		let command = cmd!("stdbuf", "-o0", "-e0", "./imu-sensor-stream-linux", "-s", "0", "--delta")
			.stdout_capture()
			.stderr_to_stdout();


		let child = command.reader().unwrap();

		let mut reader = BufReader::new(&child);

		loop {
			match kill_rx.try_recv() {
				Ok(_) | Err(TryRecvError::Disconnected) => {
					println!("Termination shell");
					break;
				}
				Err(TryRecvError::Empty) => {}
			}

			match child.try_wait() {
				Ok(Some(i)) => exit(i.status.code().unwrap()),
				Ok(None) => (),
				Err(e) => {
					println!("ERROR: CLOSING");
					dbg!(e);
					tx.send(Err(String::default())).unwrap();
					break;
				}
			}

			let mut buf = String::new();
			let bytes_read = reader.read_line(&mut buf).unwrap();

			buf.pop();

			println!("LINE!!");
			dbg!(&buf);

			
	
			match parse_stdout(&buf) {
				Some(r) => tx.send(Ok(r)),
				None => Ok(())
			};
		}
	});

	return (rx, kill_tx);
}