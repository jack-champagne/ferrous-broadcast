use std::net::SocketAddr;

#[derive(Debug)]
pub struct Message {
    from: SocketAddr,
    body: String,
}

impl Message {
    pub fn new(from: SocketAddr, body: String) -> Message {
        Message {
            from: from,
            body: body,
        }
    }
}

impl ToString for Message {
    fn to_string(&self) -> String {
        format!("{}: {}\n", self.from.to_string(), self.body)
    }
}
