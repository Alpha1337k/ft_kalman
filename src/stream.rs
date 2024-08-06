use std::{io::{BufRead, BufReader, Read, Stderr}, process::{exit, Child, Command, Stdio}, sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError}, thread::{self, JoinHandle}, time::Duration};
use std::result::Result::Ok;

use duct::{cmd, Handle, ReaderHandle};

pub fn get_stream_output(rx: &Receiver<Result<String, String>>) -> anyhow::Result<(bool, Vec<String>)> {
	let mut is_alive = true;
	let mut results = vec![];

	loop {
		let result = match rx.recv_timeout(Duration::from_millis(100)) {
			Ok(Ok(lines)) => lines,
			Ok(Err(lines)) => {
				is_alive = false;
				lines
			},
			Err(RecvTimeoutError::Timeout) => break,
			Err(RecvTimeoutError::Disconnected) => {
				is_alive = false;

				break;
			},
		};

		println!("RES: {}", result);

		if (result != "") {
			results.push(result);
		}
	}


	return anyhow::Ok((is_alive, results));
}

pub fn run_stream(path: &str) -> (Receiver<Result<String, String>>, Sender<String>) {
	let (tx, rx) = mpsc::channel::<Result<String, String>>();
	let (kill_tx, kill_rx) = mpsc::channel::<String>();

	let copy = path.to_string();

	let thd = thread::spawn(move || {
		let command = cmd!("stdbuf", "-o0", "-e0", "./imu-sensor-stream-linux", "-s", "0", "--delta")
			.stderr_capture()
			.stdout_capture();


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
					tx.send(Err(String::default())).unwrap();
					break;
				}
			}

			let mut buf = String::new();
			let line = reader.read_line(&mut buf);

			buf.pop();

			println!("LINE!!");
			dbg!(&buf);
	
			let _ = match line {
				Ok(_data) => tx.send(Ok(buf.clone())),
				Err(e) => tx.send(Err(e.to_string())),
			};
		}
	});

	return (rx, kill_tx);
}