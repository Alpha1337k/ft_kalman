use clap::Parser;
use input::{initialize_stream, read_input, write_data};
use solver::State;

mod stream;
mod input;
mod solver;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {

	#[arg(long, default_value_t = String::from("imu-sensor-stream-linux") )]
	stream: String,

	#[arg(short, long)]
	verbose: bool
}

fn main() -> anyhow::Result<()>  {
    let args = Args::parse();

	println!("WOW {} {}", args.stream, args.verbose);

	let socket = input::create_connection(4243)?;

	initialize_stream(&socket)?;

	let state = State::default();

	loop {
		let items = read_input(&socket)?;

		state.update_state(items);

		write_data(&socket, state.get_prediction().as_bytes())?;
	}


	return Ok(());
}