use std::net::UdpSocket;

pub fn create_connection(port: u16) -> anyhow::Result<UdpSocket> {
	let url = format!("localhost:{}", port);

	let socket = UdpSocket::bind(&url)?;

	return Ok(socket);
}

pub fn write_data(socket: &UdpSocket, data: &[u8]) -> anyhow::Result<()> {

	let res = socket.send_to(data, "localhost:4242")?;

	// println!("BYTES SENT: {}", res);


	return Ok(());
}


pub fn read_input(socket: &UdpSocket) -> anyhow::Result<()> {
	let mut buf: [u8; 1024] = [0;1024];

	let (res, source) = socket.recv_from(&mut buf)?;


	let reading = String::from_utf8(buf.to_vec())?;

	println!("READING {}\n---\n{}\n---", res, reading);

	return Ok(());
}
