use std::{net::SocketAddr, thread, time::Duration};

use tokio::{
    net::{tcp::OwnedWriteHalf, TcpListener, ToSocketAddrs},
    sync::mpsc::{self, Receiver, Sender},
};

use crate::{client, schema::Message};

pub async fn start_server(addr: impl ToSocketAddrs) {
    let (broadcast_tx, broadcast_rx) = mpsc::channel::<Message>(1024);
    let (connections_tx, connections_rx) = mpsc::channel::<(OwnedWriteHalf, SocketAddr)>(64);

    std::thread::spawn(move || message_broadcaster(broadcast_rx, connections_rx));

    let accept_listener = TcpListener::bind(addr).await.expect("Could not bind!");
    listen(broadcast_tx, accept_listener, connections_tx).await;
}

async fn listen(
    broadcast_tx: Sender<Message>,
    accept_listener: TcpListener,
    connections_tx: Sender<(OwnedWriteHalf, SocketAddr)>,
) {
    loop {
        let (new_stream, sock_addr) = accept_listener.accept().await.expect("Connection error");
        let (readhalf, writehalf) = new_stream.into_split();

        match connections_tx.send((writehalf, sock_addr.clone())).await {
            Ok(_) => (),
            Err(err) => println!("{}", err),
        }

        let new_client_tx = broadcast_tx.clone();
        let client_handler = client::handle_client(new_client_tx, readhalf, sock_addr.clone());
        tokio::spawn(async move { client_handler.await });
    }
}

fn message_broadcaster(
    mut rx_msgs: Receiver<Message>,
    mut rx_conns: Receiver<(OwnedWriteHalf, SocketAddr)>,
) {
    let mut msg_conns: Vec<OwnedWriteHalf> = Vec::new();
    let dur = Duration::from_micros(1_000);
    loop {
        match rx_conns.try_recv() {
            Ok((new_write_half, addr)) => {
                println!("Client {} connected", addr.to_string());
                msg_conns.push(new_write_half);
            }
            Err(_) => (),
        }

        match rx_msgs.try_recv() {
            Ok(new_message) => {
                msg_conns.retain(|conn| !conn.peer_addr().is_err());
                for conn in &msg_conns {
                    conn.try_write(&new_message.to_string().as_bytes())
                        .expect("Could not write to socket");
                }
            }
            Err(_) => (),
        };
        thread::sleep(dur);
    }
}
