use std::{
    io::{BufWriter, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        mpsc::{self, Receiver},
        Arc,
    },
};

use crate::{client, schema::Message};

pub fn start_server() {
    let accept_socket = TcpListener::bind(("localhost", 50001)).expect("Could not bind!");
    
    let (tx_msgs, rx_msgs) = mpsc::channel::<Message>();
    let (tx_conns, rx_conns) = mpsc::channel::<Arc<(TcpStream, SocketAddr)>>();

    std::thread::spawn(move || message_broadcaster(rx_msgs, rx_conns));

    loop {
        let new_client_tx = tx_msgs.clone();
        
        let clientr1 = Arc::new(accept_socket.accept().expect("Connection error"));
        let clientr2 = clientr1.clone();

        match tx_conns.send(clientr1) {
            Ok(_) => (),
            Err(err) => println!("{}", err),
        }

        std::thread::spawn(move || client::handle_client(new_client_tx, clientr2));
    }
}

fn message_broadcaster(
    rx_msgs: Receiver<Message>,
    rx_conns: Receiver<Arc<(TcpStream, SocketAddr)>>,
) {
    let mut msg_conns: Vec<Arc<(TcpStream, SocketAddr)>> = Vec::new();
    loop {
        match rx_conns.try_recv() {
            Ok(connection) => {
                println!("Client {} connected", connection.1.to_string());
                msg_conns.push(connection);
            }
            Err(_) => (),
        }

        match rx_msgs.try_recv() {
            Ok(new_message) => {
                msg_conns.retain(|conn| {
                    let (tcpstream, _addr) = conn.as_ref();
                    !tcpstream.peer_addr().is_err()
                });
                for conn in &msg_conns {
                    let (tcpstream, addr) = conn.as_ref();
                    println!("{}", addr.to_string());
                    let mut buf = BufWriter::new(tcpstream);
                    buf.write(&new_message.to_string().as_bytes())
                        .expect("Could not write to socket");
                }
            }
            Err(_) => (),
        };
    }
}
