//! TODO: code is bad, make it less bad

use clap::{Arg, ArgAction, Command};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

mod partition;

use partition::{create_partitioning, create_virtual_screens, print_virtual_screens, squareness};

async fn section_listener(
	dims: (u16, u16),
	virt_dims: (u16, u16),
	coords: (u16, u16),
	fs_url: String,
	flip_x: bool,
	flip_y: bool,
	listener: TcpListener,
) -> std::io::Result<()> {
	loop {
		let (mut stream, _) = listener.accept().await?;
		println!("got connection");

		let x_bytes = virt_dims.0.to_be_bytes();
		let y_bytes = virt_dims.1.to_be_bytes();
		let coord_bytes = [x_bytes, y_bytes].concat();

		stream.write_all(&coord_bytes).await?;

		let fs_url = fs_url.clone();
		tokio::spawn(async move {
			let mut fs_socket = match TcpStream::connect(fs_url).await {
				Ok(s) => s,
				Err(e) => {
					println!("Could not connect to Francis-Scherm\n{e}");
					return;
				},
			};

			loop {
				let mut buf = [0u8; 7];
				match stream.read_exact(&mut buf).await {
					Ok(_) => (),
					Err(_) => break,
				}

				let mut x = u16::from_be_bytes(buf[0..2].try_into().unwrap());
				let mut y = u16::from_be_bytes(buf[2..4].try_into().unwrap());

				if x >= virt_dims.0 || y >= virt_dims.1 {
					continue;
				}

				x += coords.0;
				if flip_x {
					x = dims.0 - x;
				}

				y += coords.1;
				if flip_y {
					y = dims.1 - y;
				}

				let x_bytes = x.to_be_bytes();
				let y_bytes = y.to_be_bytes();

				buf[0] = x_bytes[0];
				buf[1] = x_bytes[1];
				buf[2] = y_bytes[0];
				buf[3] = y_bytes[1];

				let _ = fs_socket.write_all(&buf).await;
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
			Arg::new("fs_url")
				.short('u')
				.long("fs-url")
				.help("The URL of the Francis-Scherm TCP socket")
				.action(ArgAction::Set)
				.value_parser(clap::value_parser!(String))
				.required(true),
		)
		.arg(
			Arg::new("flip_x")
				.long("flip-x")
				.help("Flip the image along the x axis")
				.action(ArgAction::SetTrue),
		)
		.arg(
			Arg::new("flip_y")
				.long("flip-y")
				.help("Flip the image along the y axis")
				.action(ArgAction::SetTrue),
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
	let fs_url = matches.get_one::<String>("fs_url").unwrap();
	let sections = *matches.get_one::<usize>("sections").unwrap();
	let flip_x = matches.get_flag("flip_x");
	let flip_y = matches.get_flag("flip_y");

	println!("Querying dimensions from Francis-Scherm...");
	let mut fs_socket = TcpStream::connect(fs_url).await?;
	let mut size_buf = [0; 4];
	fs_socket.read_exact(&mut size_buf).await?;
	drop(fs_socket);

	let width = u16::from_be_bytes(size_buf[0..2].try_into().unwrap()) as usize;
	let height = u16::from_be_bytes(size_buf[2..4].try_into().unwrap()) as usize;

	println!("Partitioning {width}x{height} screen into {sections} sections...");
	let partitioning = create_partitioning(width, height, sections);
	println!("Found partitioning with squareness {}", squareness(&partitioning));

	let virtual_screens = create_virtual_screens(&partitioning);
	println!("Virtual screens:");
	print_virtual_screens(&partitioning, &virtual_screens);

	println!("Creating virtual screen sockets...");
	let mut handles = vec![];
	for i in 0..sections {
		let sect_dims = partitioning
			.iter()
			.flatten()
			.nth(i)
			.map(|p| (p.0 as u16, p.1 as u16))
			.unwrap()
			.to_owned();
		let sect_coords = virtual_screens
			.iter()
			.flatten()
			.nth(i)
			.map(|p| (p.0 as u16, p.1 as u16))
			.unwrap()
			.to_owned();
		let socket_url = format!("0.0.0.0:{}", 8000 + i);
		let listener = TcpListener::bind(&socket_url).await?;

		let handle = tokio::spawn(section_listener(
			(width as u16, height as u16),
			sect_dims,
			sect_coords,
			fs_url.to_owned(),
			flip_x,
			flip_y,
			listener,
		));
		handles.push(handle);

		println!("Created socket on {}", &socket_url);
		println!("Dimensions ({}x{})", sect_dims.0, sect_dims.1);
		println!("Coords ({}; {})\n", sect_coords.0, sect_coords.1);
	}

	println!("Running!");

	for handle in handles {
		handle.await??;
	}

	Ok(())
}
