use clap::Parser;
use input::{read_input, write_data};

mod stream;
mod input;

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

	write_data(&socket, "READY".as_bytes())?;

	loop {
		read_input(&socket)?;
		write_data(&socket, "0.0 0.0 0.0".as_bytes())?;
	}


	return Ok(());
}