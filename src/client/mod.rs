use std::net::SocketAddr;

use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    net::tcp::OwnedReadHalf,
    sync::mpsc::Sender,
};

use crate::schema::Message;

pub async fn handle_client(broadcast: Sender<Message>, read_half: OwnedReadHalf, addr: SocketAddr) {
    let mut buf = BufReader::new(read_half);
    loop {
        let body = match get_message_body(&mut buf).await {
            Ok(msg) => msg,
            Err(err) => match err.kind() {
                io::ErrorKind::BrokenPipe => break,
                io::ErrorKind::InvalidData => continue,
                _ => continue,
            },
        };

        let new_message = Message::new(addr, body);
        print!("Recieved: {}", new_message.to_string());

        match broadcast.send(new_message).await {
            Ok(_) => continue,
            Err(err) => broadcast
                .blocking_send(err.0)
                .expect("blocking send failed!"),
        };
    }
    println!("Client {} disconnected", addr.to_string());
}

async fn get_message_body(
    client_reader: &mut BufReader<OwnedReadHalf>,
) -> Result<String, io::Error> {
    let mut buf = String::new();
    match client_reader.read_line(&mut buf).await {
        Ok(0) => {
            return Err(io::Error::from(io::ErrorKind::BrokenPipe));
        }
        Ok(_bytes) => {
            let buf = buf.trim().to_owned();
            match buf.len() {
                0 => return Err(std::io::Error::from(io::ErrorKind::InvalidData)),
                _ => return Ok(buf),
            };
        }
        Err(err) => return Err(err),
    };
}
