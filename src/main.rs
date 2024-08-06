use std::{fs, io::Write, process::exit, thread::sleep, time::Duration};

use anyhow::{Error, Ok};
use clap::{Parser, Subcommand};
use input::{initialize_stream, read_input, write_data};
use solver::State;
use stream::{get_stream_output, run_stream};

mod stream;
mod input;
mod solver;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {

	#[arg(long, default_value_t = String::from("./imu-sensor-stream-linux") )]
	stream: String,

	#[arg(short, long)]
	verbose: bool,

	#[command(subcommand)]
	command: Option<Commands>
}

#[derive(Debug)]
#[derive(Subcommand)]
enum Commands {
	MakeDataset {
		#[arg(long, short)]
		output: String
	}
}

fn make_dataset(args: &Args, output: &String) -> anyhow::Result<()> {
	let mut file = fs::File::create(output)?;

	let headers = ["index", "TIMESTAMP", "TRUE_POSITION", "SPEED", "ACCELERATION", "DIRECTION"].join(",") + "\n";

	file.write(headers.as_bytes())?;

	let (rx, tx) = run_stream(&args.stream);

	sleep(Duration::from_millis(200));

	let mut output = get_stream_output(&rx)?;

	dbg!(&output);

	if (output.0 == false) {
		println!("ft_kalman: stream exited:\n> {}", output.1.join("\n> "));
		return Err(Error::msg("Stream Exited"));
	}

	output = get_stream_output(&rx)?;

	let socket = input::create_connection(4243)?;

	let state = State::default();
	initialize_stream(&socket)?;

	output = get_stream_output(&rx)?;

	dbg!(output);

	let mut idx = 0;

	loop {
		println!("-- {} --", idx);

		let items = read_input(&socket)?;
		dbg!(&items);

		state.update_state(items);
		write_data(&socket, state.get_prediction().as_bytes())?;


		println!("WE IN ERE");

		output = get_stream_output(&rx)?;
	
		if (output.0 == false) {
			println!("ft_kalman: stream exited: > {}", output.1.join("\n> "));
			return Err(Error::msg("Stream Exited"));
		}

		println!("OUT:\n> {}", output.1.join("\n> "));

		sleep(Duration::from_millis(10));

		println!("-- -- --");

		idx += 1;
	}
	
	tx.send("KILL".to_string())?;

	sleep(Duration::from_millis(1200));

	Ok(())
}

fn run_kalman(args: &Args) -> anyhow::Result<()> {
	let socket = input::create_connection(4243)?;

	initialize_stream(&socket)?;

	let state = State::default();

	loop {
		let items = read_input(&socket)?;

		state.update_state(items);

		write_data(&socket, state.get_prediction().as_bytes())?;
	}
}

fn main() -> anyhow::Result<()>  {
    let args = Args::parse();

	match &args.command {
		Some(Commands::MakeDataset { output }) => make_dataset(&args, &output),
		None => run_kalman(&args)
	}
}