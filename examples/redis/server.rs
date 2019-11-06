use futures::TryStreamExt;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

type Error = Box<dyn std::error::Error>;

#[derive(Clone, Debug, Default)]
struct Database {}

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
    async fn handle_connection(
        &mut self,
        mut conn: TcpStream,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (r, w) = conn.split();

        let mut lines = BufReader::new(r).lines();

        let arg = lines.try_next().await?;

        Ok(())
    }
}

// async fn parse_request(conn: &mut TcpStream) -> Result<String, Error> {}
