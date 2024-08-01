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
            Self::System => "system",
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
    connection: Client,
}
/*I don't have a definitive list of "blocked" words, but I can give you an idea of the types of words and phrases that I'm programmed to avoid or treat with caution. Keep in mind that this is not an exhaustive list, and my ability to recognize and respond to sensitive language is an ongoing process.

Here are some examples of words and phrases that I'm designed to handle with care:

**Profanity and strong language**: I avoid using words that are commonly considered profane or strong. This includes words that might be coded or used in a way that's intentionally inflammatory.

**Derogatory terms**: I'm programmed to recognize and avoid words that are used to demean or degrade individuals or groups based on their identity, culture, or characteristics.

**Hate speech**: I'm designed to detect and respond to language that promotes hatred, intolerance, or violence towards specific groups or individuals.

**Graphic or disturbing content**: I won't discuss or describe graphic violence, self-harm, or other disturbing topics in a way that might be upsetting or triggering.

**Sensitive topics**: I'm cautious when discussing sensitive topics like mental health, trauma, or abuse. I'll try to provide helpful and supportive responses while avoiding triggering or explicit language.

Some specific examples of words or phrases that I'm programmed to avoid include:

* Racial or ethnic slurs
* Homophobic or transphobic language
* Sexist or misogynistic terms
* Derogatory terms for people with disabilities
* Pornographic or explicit descriptions
* Hate group names or ideologies
* Language that promotes violence or harm
* Explicit or graphic descriptions of trauma or abuse

Remember, my purpose is to provide helpful and informative responses while maintaining a respectful and inclusive environment. If I ever respond to a question or prompt in a way that makes you uncomfortable, please let me know, and I'll do my best to correct the situation.*/
impl Context {
    pub async fn new() -> Self {
        let connection = db_connect().await;
        Self {
            messages: HashMap::new(),
            connection,
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
    let (client, connection) = connect(
        "dbname=context hostaddr=0.0.0.0 port=5433 user=postgres password=postgres",
        NoTls,
    )
    .await
    .unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    client
}
