use futures::{Stream, TryStreamExt};
use redis_protocol::types::Frame;
use std::{collections::HashMap, io, sync::Arc};
use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Clone, Debug, Default)]
struct Database {
    db: Arc<Mutex<HashMap<String, Frame>>>,
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

    let db = Database::default();

    loop {
        let (connection, _addr) = listener.accept().await?;

        let mut db = db.clone();

        tokio::spawn(async move {
            if let Err(e) = db.handle_connection(connection).await {
                eprintln!("Connection Error: {}", e);
            }
        });
    }
}

impl Database {
    async fn handle_connection(&mut self, mut conn: TcpStream) -> Result<(), Error> {
        let (r, mut w) = conn.split();

        let buf = BufReader::new(r);
        let mut lines = buf.lines();

        if let Some(line) = lines.next_line().await? {
            match &line[0..1] {
                "*" => {
                    let mut full_message = line.clone();

                    full_message += "\r\n";

                    let array_len = line[1..].parse::<usize>()?;

                    for _ in 0..array_len {
                        let type_line = lines.next_line().await?.unwrap();
                        let body_line = lines.next_line().await?.unwrap();

                        full_message += &type_line;
                        full_message += "\r\n";

                        full_message += &body_line;
                        full_message += "\r\n";
                    }

                    let frame = redis_protocol::decode::decode(full_message.as_bytes()).unwrap();

                    println!("decoded frame {:?}", frame);

                    if let (Some(frame), _) = frame {
                        match self.handle_request(frame).await {
                            Ok(Response::Ok) => w.write_all("+Ok\r\n".as_bytes()).await?,
                            Ok(Response::Frame(f)) => {
                                let mut buf = vec![0u8; 1024];

                                redis_protocol::encode::encode(&mut buf, &f).unwrap();

                                w.write_all(&buf[..]).await?;
                            }
                            Ok(Response::Error(e)) => unimplemented!(),
                            Err(_e) => {
                                unimplemented!("implement error message");
                            }
                        }
                    }
                }
                _ => unimplemented!("Only array's are supported"),
            }
        } else {
            eprintln!("Connection closed early");
        }

        Ok(())
    }

    async fn handle_request(&mut self, frame: Frame) -> Result<Response, Error> {
        if let Frame::Array(frames) = frame {
            let mut frames = frames.into_iter();

            if let Some(Frame::BulkString(s)) = frames.next() {
                let cmd = String::from_utf8_lossy(&s[..]).into_owned();

                match cmd.to_uppercase().as_str() {
                    "GET" => {
                        let key = if let Some(Frame::BulkString(s)) = frames.next() {
                            String::from_utf8_lossy(&s[..]).into_owned()
                        } else {
                            unimplemented!("expected bulkstring for key");
                        };

                        let db = self.db.lock().await;

                        if let Some(frame) = db.get(&key) {
                            return Ok(Response::Frame(frame.clone()));
                        } else {
                            unimplemented!("key does not exist.")
                        }
                    }

                    "SET" => {
                        let key = if let Some(Frame::BulkString(s)) = frames.next() {
                            String::from_utf8_lossy(&s[..]).into_owned()
                        } else {
                            unimplemented!("expected bulkstring for key");
                        };

                        let value = if let Some(frame) = frames.next() {
                            frame
                        } else {
                            unimplemented!("PUT expects a value");
                        };

                        let mut db = self.db.lock().await;

                        db.insert(key, value);

                        return Ok(Response::Ok);
                    }
                    _ => unimplemented!("command not supported"),
                }
            } else {
                unimplemented!("Expected array with atleast one item and of type BulkString");
            }
        } else {
            unimplemented!("only outter array frame types are accepted.")
        }
    }
}

enum Response {
    Ok,
    Frame(Frame),
    Error(String),
}
