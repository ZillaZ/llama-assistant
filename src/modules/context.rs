use std::{collections::HashMap, fmt::Display};
use tokio_postgres::{connect, Client, NoTls};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Entity {
    User,
    Assistant,
    System,
}

impl Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = match *self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::System => "system"
        };
        f.write_str(repr)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    pub role: Entity,
    pub content: String,
}

#[derive(Debug)]
pub struct Context {
    messages: HashMap<String, Vec<Message>>,
    connection: Client
}

impl Context {
    pub async fn new() -> Self {
        let connection = db_connect().await;
        Self {
            messages: HashMap::new(),
            connection
        }
    }

    pub fn new_message(&mut self, id: String, message: Message) {
        let entry = self.messages.entry(id).or_insert(Vec::new());
        entry.push(message);
    }

    pub fn get_messages(&self, id: &String) -> Vec<Message> {
        self.messages.get(id).unwrap().to_vec()
    }

}
pub async fn db_connect() -> Client {
    let (client, connection) = connect("dbname=context hostaddr=0.0.0.0 port=5433 user=postgres password=postgres", NoTls)
        .await
        .unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    client
}
