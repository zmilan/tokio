use redis_protocol::types::Frame;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{channel, UnboundedSender},
        Mutex,
    },
};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

enum Response {
    Ok,
    Frame(Frame),
    Subscribed(Vec<String>),
    Error(String),
}

#[derive(Debug, Default)]
struct Kv {
    kv: HashMap<String, Frame>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Parse the address we're going to run this server on
    // and set up our TCP listener to accept connections.
    let addr = std::env::args()
        .nth(1)
        .unwrap_or("127.0.0.1:6379".to_string());

    let mut listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    let db = Arc::new(Mutex::new(Kv::default()));

    let (publish_tx, publish_rx) = mpsc::unbounded_channel();
    let (subscribe_tx, subscribe_rx) = watch::channel(None);

    let pubsub = Pubsub {
        publish: publish_tx,
        subscribe: subscribe_rx,
    };

    tokio::spawn(async move {
        while let Some(msg) = publish_rx.recv().await {
            subscribe_tx.broadcast(Some(msg)).unwrap();
        }
    });

    loop {
        let (stream, _addr) = listener.accept().await?;

        let mut connection = Connection::new(stream, db.clone(), pubsub.clone());

        tokio::spawn(async move {
            if let Err(e) = connection.process().await {
                eprintln!("Connection Error: {}", e);
            }
        })
    }
}

impl Kv {
    fn get(&mut self, key: String) -> Response {
        if let Some(frame) = self.kv.get(&key) {
            Response::Frame(frame.clone())
        } else {
            Response::Frame(Frame::Null)
        }
    }

    fn put(&mut self, key: String, value: Frame) -> Response {
        self.kv.insert(key, value);

        Response::Ok
    }

    fn publish(&mut self, channel: String, value: Frame) -> Response {
        if let Some(clients) = self.watchers.get_mut(&channel) {
            let mut num_sent = 0;

            let mut clients_to_remove = Vec::new();

            for (i, client) in clients.iter().enumerate() {
                if let Err(_) = client.send(value.clone()) {
                    clients_to_remove.push(i);
                } else {
                    num_sent += 1;
                }
            }

            for i in clients_to_remove {
                clients.remove(i);
            }

            Response::Frame(Frame::Integer(num_sent))
        } else {
            Response::Frame(Frame::Integer(0))
        }
    }
}

use tokio::io::AsyncReadExt;
use tokio::sync::watch;

#[derive(Debug)]
struct Connection {
    conn: TcpStream,
    kv: Arc<Mutex<Kv>>,
    pubsub: Pubsub,
    state: State,

    buf: Vec<u8>,
    buf_offset: usize,
}

#[derive(Debug)]
enum State {
    Normal,
    Subscribed(Vec<String>),
}

impl Connection {
    pub fn new(conn: TcpStream, kv: Arc<Mutex<Kv>>, pubsub: Pubsub) -> Self {
        Self {
            conn,
            kv,
            pubsub,
            state: State::Normal,
            buf: vec![0u8; 1024],
            buf_offset: 0,
        }
    }

    pub async fn process(&mut self) -> Result<(), Error> {
        match &mut self.state {
            State::Normal => {
                while let Some(frame) = self.read_frame().await? {
                    match self.handle_request(frame).await? {
                        Response::Ok => self.conn.write_all("+OK\r\n".as_bytes()).await?,
                        Response::Frame(f) => {
                            let mut buf = vec![0u8; 1024];

                            redis_protocol::encode::encode(&mut buf[..], &f).unwrap();

                            self.conn.write_all(&buf[..]).await?;
                        }
                        Response::Subscribed(channels) => {
                            let sub = Frame::BulkString("subscribed".to_string().into_bytes());
                            let channel = Frame::BulkString()
                        }
                    }
                }
            }

            State::Subscribed(_channels) => {
                futures::select! {
                    _ = self.read_frame() => {
                        // TODO: handle subscribe/unsubscribe commands
                    }

                    _ = self.pubsub.recv() => {
                        // TODO: incoming frame write to channel
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_request(&mut self, frame: Frame) -> Result<Response, Error> {
        if let Frame::Array(frames) = frame {
            let mut frames = frames.into_iter();

            if let Some(Frame::BulkString(s)) = frames.next() {
                let cmd = String::from_utf8(s)?;

                match cmd.to_lowercase().as_str() {
                    "command" => Ok(Response::Ok),

                    "get" => {
                        let response = if let Some(Frame::BulkString(s)) = frames.next() {
                            let key = String::from_utf8(s)?;

                            self.kv.lock().await.get(key)
                        } else {
                            Response::Error("wrong number of arguments for `get` command".into())
                        };

                        Ok(response)
                    }

                    "set" => {
                        let response = match (frames.next(), frames.next()) {
                            (Some(Frame::BulkString(key)), Some(value)) => {
                                let key = String::from_utf8(key)?;

                                self.kv.lock().await.put(key, value)
                            }

                            _ => Response::Error(
                                "wrong number of arguments for 'set' command".into(),
                            ),
                        };

                        Ok(response)
                    }

                    "publish" => {
                        let response = match (frames.next(), frames.next()) {
                            (Some(Frame::BulkString(channel)), Some(value)) => {
                                let channel = String::from_utf8(channel)?;

                                // TODO: get number of clients published too
                                self.pubsub.publish(channel, value);
                                Response::Frame(Frame::Integer(0))
                            }

                            _ => unimplemented!("error case of wrong command"),
                        };

                        Ok(response)
                    }

                    "subscribe" => {
                        let mut channels = Vec::new();

                        while let Some(Frame::BulkString(s)) = frames.next() {
                            let channel = String::from_utf8(s)?;
                            channels.push(channel);
                        }

                        self.state = State::Subscribed(channels.clone());
                        Ok(Response::Subscribed(channels))
                    }

                    cmd => {
                        let mut args = Vec::new();

                        while let Some(frame) = frames.next() {
                            let arg = match frame {
                                Frame::BulkString(s) => String::from_utf8(s)?,
                                Frame::Integer(i) => format!("{}", i),
                                _ => unimplemented!("format arg"),
                            };

                            args.push(format!("`{}`", arg));
                        }

                        let error = format!(
                            "ERR unknown command `{}`, with args beginning with: {}",
                            cmd,
                            args.join(", ")
                        );

                        Ok(Response::Error(error))
                    }
                }
            } else {
                Ok(Response::Error(
                    "ERR only array frame types accepted.".into(),
                ))
            }
        } else {
            Ok(Response::Error(
                "ERR only array frame types accepted.".into(),
            ))
        }
    }

    async fn read_frame(&mut self) -> Result<Option<Frame>, Error> {
        loop {
            let amt = self.conn.read(&mut self.buf[self.buf_offset..]).await?;

            self.buf_offset += amt;

            if amt == 0 {
                return Ok(None);
            }

            if let (Some(frame), _) =
                redis_protocol::decode::decode(&self.buf[..self.buf_offset + 1]).unwrap()
            {
                self.buf_offset = 0;
                return Ok(Some(frame));
            }
        }
    }
}

use tokio::sync::mpsc;

#[derive(Debug, Clone)]
struct Pubsub {
    publish: mpsc::UnboundedSender<(String, Frame)>,
    subscribe: watch::Receiver<Option<(String, Frame)>>,
}

impl Pubsub {
    pub async fn recv(&mut self) -> Option<(String, Frame)> {
        self.subscribe.recv().await.unwrap()
    }

    pub async fn publish(&mut self, channel: String, value: Frame) {
        self.publish.send((channel, value)).unwrap()
    }
}

// impl Server {
//     async fn handle_connection(&mut self, mut conn: TcpStream) -> Result<(), Error> {
//         let (r, mut w) = conn.split();

//         let buf = BufReader::new(r);
//         let mut lines = buf.lines();

//         while let Some(line) = lines.next_line().await? {
//             match &line[0..1] {
//                 "*" => {
//                     let mut full_message = line.clone();

//                     full_message += "\r\n";

//                     let array_len = line[1..].parse::<usize>()?;

//                     for _ in 0..array_len {
//                         let type_line = lines.next_line().await?.unwrap();
//                         let body_line = lines.next_line().await?.unwrap();

//                         full_message += &type_line;
//                         full_message += "\r\n";

//                         full_message += &body_line;
//                         full_message += "\r\n";
//                     }

//                     let frame = redis_protocol::decode::decode(full_message.as_bytes()).unwrap();

//                     if let (Some(frame), _) = frame {
//                         match self.handle_request(frame).await? {
//                             Response::Ok => w.write_all("+OK\r\n".as_bytes()).await?,
//                             Response::Frame(f) => {
//                                 let mut buf = vec![0u8; 1024];

//                                 redis_protocol::encode::encode(&mut buf, &f).unwrap();

//                                 w.write_all(&buf[..]).await?;
//                             }
//                             Response::Error(e) => {
//                                 let error = format!("-{}\r\n", e);
//                                 w.write_all(error.as_bytes()).await?;
//                             }
//                         }
//                     }
//                 }
//                 _ => {
//                     w.write_all("-only 'set' and 'get' commands are supported\r\n".as_bytes())
//                         .await?
//                 }
//             }
//         }

//         Ok(())
//     }

// }
