use std::{fs, io::Write, process::exit, thread::sleep, time::{Duration, Instant}};

use anyhow::{Error, Ok};
use clap::{Parser, Subcommand};
use input::{initialize_stream, read_input, write_data, ResultType, StreamResult};
use solver::{State, Vec3};
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

fn format_payload_item(res: Option<&StreamResult>) -> String {
	if (res.is_none()) {
		return String::new();
	}
	let items = &res.unwrap().payload_items;

	if (items.len() == 1) {
		let item = items.first().unwrap();
		return item.to_string();
	}


	return format!("{},{},{}", items[0].to_string(), items[1].to_string(), items[2].to_string());
}

fn make_dataset(args: &Args, output: &String) -> anyhow::Result<()> {
	let mut file = fs::File::create(output)?;

	let headers = ["index", "TIMESTAMP", "TRUE_POSITION_X", "TRUE_POSITION_Y", "TRUE_POSITION_Z", "SPEED", "ACCELERATION_X", "ACCELERATION_Y", "ACCELERATION_Z", "DIRECTION_X", "DIRECTION_Y", "DIRECTION_Z"].join(",") + "\n";

	file.write(headers.as_bytes())?;

	let (rx, tx) = run_stream(&args.stream);

	sleep(Duration::from_millis(200));

	let mut output = get_stream_output(&rx)?;

	dbg!(&output);

	if (output.0 == false) {
		println!("ft_kalman: stream exited: {:?}", output.1);
		return Err(Error::msg("Stream Exited"));
	}

	output = get_stream_output(&rx)?;

	let socket = input::create_connection(4243)?;

	let mut state = State::default();

	state.position = Vec3 {
		x: 1.8,
		y: -4.7,
		z: 0.5
	};

	initialize_stream(&socket)?;

	output = get_stream_output(&rx)?;

	dbg!(output);

	let mut idx: i32 = 0;

	let now = Instant::now();

	let mut timestamp = 0;

	loop {
		println!("-- {} -- {}", idx, now.elapsed().as_millis());

		let items = read_input(&socket)?;
		dbg!(&items);

		state.update_state(&items);
		write_data(&socket, state.get_prediction().as_bytes())?;


		println!("WE IN ERE");

		if let Some(truth) = items.iter().find(|x| x.payload_type == ResultType::TruePosition) {
			state.position = Vec3 {
				x: truth.payload_items[0],
				y: truth.payload_items[1],
				z: truth.payload_items[2],
			}
		}

		match get_stream_output(&rx) {
			std::result::Result::Ok(d) => {
				if (d.0 == false) {
					println!("ft_kalman: stream exited: > {:?}", &d.1);
					return Err(Error::msg("Stream Exited"));
				}
			},
			Err(l) => {
				dbg!(&l);
				return Err(l);
			}
		}

		println!("-- -- --");

		let true_pos = items.iter().find(|x| x.payload_type == ResultType::TruePosition);
		let speed = items.iter().find(|x| x.payload_type == ResultType::Speed);
		let accel = items.iter().find(|x| x.payload_type == ResultType::Acceleration);
		let direction = items.iter().find(|x| x.payload_type == ResultType::Direction);


		file.write(format!("{},{},{},{},{},{}\n", 
			&idx,
			timestamp,
			format_payload_item(true_pos),
			format_payload_item(speed),
			format_payload_item(accel),
			format_payload_item(direction),
		).as_bytes())?;

		timestamp += 10;

		idx += 1;

		// if (idx == 200) {
		// 	state.position = Vec3 {
		// 		x: 3.797511604733362,
		// 		y: -1.2171945819828736,
		// 		z: 0.5
		// 	}
		// }

		// if (idx == 400) {
		// 	state.position = Vec3 {
		// 		x: 8.203794545537798,
		// 		y: -1.2171945819828736,
		// 		z: 0.5
		// 	}
		// }

		if (idx > 30_000) {
			break;
		}
	}
	
	tx.send("KILL".to_string())?;

	sleep(Duration::from_millis(1200));

	Ok(())
}

fn run_kalman(args: &Args) -> anyhow::Result<()> {
	let socket = input::create_connection(4243)?;

	initialize_stream(&socket)?;

	let state = State::default();

	let mut reader = csv::Reader::from_path("out.csv")?;

	let mut records = reader.records();
	let mut i = 0;

	loop {
		// println!("-- -- --");

		let items = read_input(&socket)?;

		state.update_state(&items, );

		let item= records.next().unwrap()?;

		let s = format!("{} {} {}", item[2].to_string(), item[3].to_string(), item[4].to_string());

		write_data(&socket, s.as_bytes())?;

		// println!("-- -- --");
		i += 1;
	}
}

fn main() -> anyhow::Result<()>  {
    let args = Args::parse();

	match &args.command {
		Some(Commands::MakeDataset { output }) => make_dataset(&args, &output),
		None => run_kalman(&args)
	}
}