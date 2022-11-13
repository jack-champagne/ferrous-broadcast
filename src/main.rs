use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        mpsc::{self, Sender},
        Arc,
    },
};

#[derive(Debug)]
struct Message {
    from: String,
    body: String,
}

fn main() {
    let accept_socket = TcpListener::bind(("192.168.0.26", 50001)).expect("Could not bind!");
    let (tx_msgs, rx_msgs) = mpsc::channel::<Message>();
    let (tx_conns, rx_conns) = mpsc::channel::<Arc<(TcpStream, SocketAddr)>>();

    std::thread::spawn(move || {
        let mut msg_conns: Vec<Arc<(TcpStream, SocketAddr)>> = Vec::new();
        loop {
            match rx_conns.try_recv() {
                Ok(connection) => {
                    msg_conns.push(connection);
                    dbg!(&msg_conns);
                }
                Err(_) => (),
            }

            match rx_msgs.try_recv() {
                Ok(new_message) => {
                    dbg!(&new_message);
                    dbg!(&msg_conns);
                    for conn in &msg_conns {
                        let (tcpstream, _addr) = conn.as_ref();
                        let mut buf = BufWriter::new(tcpstream);
                        let out = format!("{}: {}\n", new_message.from, new_message.body);
                        buf.write(&out.as_bytes()).expect("Could not write to buf");
                    }
                }
                Err(_) => (),
            };
        }
    });

    loop {
        let tx1 = tx_msgs.clone();
        let clientr1 = Arc::new(accept_socket.accept().expect("Connection error"));
        let clientr2 = clientr1.clone();

        match tx_conns.send(clientr1) {
            Ok(_) => (),
            Err(err) => println!("{}", err),
        }

        std::thread::spawn(move || handle_client(tx1, clientr2));
    }
}

fn handle_client(tx: Sender<Message>, client_arc: Arc<(TcpStream, SocketAddr)>) {
    let (tcpstream, addr) = client_arc.as_ref();
    let mut client_reader: BufReader<&TcpStream> = BufReader::new(tcpstream);
    loop {
        // make owned string, wait for message
        let mut buf = String::new();
        if client_reader
            .read_line(&mut buf)
            .expect("Error reading from buffer")
            == 0
        {
            return;
        }

        // Sanitize recieved
        buf = buf.trim_end().to_owned();

        // Pack message into struct Message
        let new_message = Message {
            from: addr.to_string(),
            body: buf,
        };

        // send using channel up to main thread
        match tx.send(new_message) {
            Err(_) => todo!(),
            _ => (),
        };
    }
}
