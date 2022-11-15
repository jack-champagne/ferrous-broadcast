use std::{
    io::{self, BufRead, BufReader},
    net::{SocketAddr, TcpStream},
    sync::{mpsc::Sender, Arc},
};

use crate::schema::Message;

pub fn handle_client(tx: Sender<Message>, client_arc: Arc<(TcpStream, SocketAddr)>) {
    let (tcpstream, addr) = client_arc.as_ref();
    let mut client_reader: BufReader<&TcpStream> = BufReader::new(tcpstream);
    loop {
        let message_body = match get_message_body(&mut client_reader) {
            Ok(msg) => msg,
            Err(_) => break,
        };

        let new_message = Message::new(*addr, message_body);
        print!("{}", new_message.to_string());

        match tx.send(new_message) {
            Ok(_) => (),
            Err(_) => continue,
        }
    }
    println!("Client {} disconnected", addr.to_string());
}

fn get_message_body(client_reader: &mut BufReader<&TcpStream>) -> Result<String, io::Error> {
    let mut buf = String::new();
    match client_reader.read_line(&mut buf) {
        Ok(bytes) => {
            if bytes == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Client connection terminated",
                ));
            }
            buf = buf.trim_end().to_owned();
            Ok(buf)
        }
        Err(err) => return Err(err),
    }
}
