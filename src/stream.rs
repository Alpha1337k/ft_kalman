use std::process::{Command, Stdio};

fn run_stream(path: &str) {

	let mut command = Command::new(path);

	command.arg("--debug");

}