//! TODO: code is bad, make it less bad

use clap::{Arg, ArgAction, Command};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{self, Sender};

mod partition;

use partition::{
	create_partitioning,
	create_virtual_screens,
	print_partitioning,
	print_virtual_screens,
	squareness,
};

const FS_URL: &str = "10.1.0.193:8000";
const FS_WIDTH: u16 = 1680;
const FS_HEIGHT: u16 = 1050;

async fn section_listener(
	dims: (usize, usize),
	coords: (usize, usize),
	listener: TcpListener,
	tx: Sender<Vec<u8>>,
) -> std::io::Result<()> {
	loop {
		let (mut stream, _) = listener.accept().await?;
		println!("got connection");
		let tx = tx.clone();

		tokio::spawn(async move {
			loop {
				let mut buf = [0u8; 7];
				match stream.read_exact(&mut buf).await {
					Ok(_) => (),
					Err(_) => break,
				}

				let mut x = u16::from_be_bytes(buf[0..2].try_into().unwrap());
				let mut y = u16::from_be_bytes(buf[2..4].try_into().unwrap());

				if x as usize >= dims.0 || y as usize >= dims.1 {
					continue;
				}

				x += coords.clone().0 as u16;
				// x = FS_WIDTH - x;

				y += coords.clone().1 as u16;
				// y = FS_HEIGHT - y;

				let x_bytes = x.to_be_bytes();
				let y_bytes = y.to_be_bytes();

				buf[0] = x_bytes[0];
				buf[1] = x_bytes[1];
				buf[2] = y_bytes[0];
				buf[3] = y_bytes[1];

				let _ = tx.send(buf.to_vec()).await;
			}
		});
	}
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
	let matches = Command::new(env!("CARGO_PKG_NAME"))
		.version(env!("CARGO_PKG_VERSION"))
		.author(env!("CARGO_PKG_AUTHORS"))
		.about(env!("CARGO_PKG_DESCRIPTION"))
		.arg_required_else_help(true)
		.arg(
			Arg::new("width")
				.short('x')
				.long("width")
				.help("The total width of the Francis-Scherm servers screen")
				.action(ArgAction::Set)
				.value_parser(clap::value_parser!(usize))
				.required(true),
		)
		.arg(
			Arg::new("height")
				.short('y')
				.long("height")
				.help("The total height of the Francis-Scherm servers screen")
				.action(ArgAction::Set)
				.value_parser(clap::value_parser!(usize))
				.required(true),
		)
		.arg(
			Arg::new("sections")
				.short('n')
				.long("sections")
				.help("The amount of virtual screen sections to segment the display into")
				.action(ArgAction::Set)
				.value_parser(clap::value_parser!(usize))
				.required(true),
		)
		.get_matches();

	// Unwraps are safe as arguments are required
	let width = *matches.get_one::<usize>("width").unwrap();
	if width as u16 > FS_WIDTH {
		panic!(
			"Virtual width cannot be larger then the actual width ({FS_WIDTH}) of Francis scherm"
		);
	}
	let height = *matches.get_one::<usize>("height").unwrap();
	if height as u16 > FS_HEIGHT {
		panic!(
			"Virtual height cannot be larger then the actual height ({FS_HEIGHT}) of Francis \
			 scherm"
		);
	}
	let sections = *matches.get_one::<usize>("sections").unwrap();

	println!("Partitioning {width}x{height} screen into {sections} sections...");
	let partitioning = create_partitioning(width, height, sections);
	println!("Found partitioning with squareness {}:", squareness(&partitioning));
	print_partitioning(&partitioning);

	let virtual_screens = create_virtual_screens(&partitioning);
	println!("Virtual screens:");
	print_virtual_screens(&partitioning, &virtual_screens);

	let (tx, mut rx) = mpsc::channel::<Vec<u8>>(128);

	println!("Creating virtual screen sockets...");
	for i in 0..sections {
		let sect_dims = partitioning.iter().flatten().nth(i).unwrap().to_owned();
		let sect_coords = virtual_screens.iter().flatten().nth(i).unwrap().to_owned();
		let socket_url = format!("0.0.0.0:{}", 8000 + i);
		let listener = TcpListener::bind(&socket_url).await?;

		println!("listener {i}: {:?} {:?}", sect_dims, sect_coords);
		tokio::spawn(section_listener(sect_dims, sect_coords, listener, tx.clone()));

		println!("Created socket for {}", &socket_url);
	}

	println!("Connecting to Francis-Scherm ({})", FS_URL);
	let mut fs_socket = TcpStream::connect(FS_URL).await?;

	println!("Running!");

	loop {
		let msg = rx.recv().await;
		if msg.is_none() {
			break;
		}

		fs_socket.write_all(msg.unwrap().as_ref()).await?;
	}

	Ok(())
}
