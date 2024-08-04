use std::process::{Command};

fn run_stream(path: &str) {

	let mut command = Command::new(path);

	command.arg("--debug");

}