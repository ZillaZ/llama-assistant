use crate::modules::context::{Context, Message};
use crate::modules::env::Environment;
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio::{
    io::AsyncReadExt,
    net::TcpStream,
    sync::mpsc::{UnboundedReceiver as Receiver, UnboundedSender as Sender},
};

use super::context::Entity;

#[derive(Deserialize, Serialize)]
pub struct Usage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
    pub prompt_time: f32,
    pub completion_time: f32,
    pub total_time: f32,
}

#[derive(Deserialize, Serialize)]
pub struct Choice {
    pub index: i32,
    pub message: ApiMessage,
    pub finish_reason: String,
    pub logprobs: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct ApiResponse {
    pub id: String,
    pub object: String,
    pub created: i32,
    pub model: String,
    pub system_configuration: Option<String>,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Deserialize, Serialize)]
pub struct Messages {
    pub messages: Vec<ApiMessage>,
    pub model: String,
}

impl Display for Messages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let messages = self
            .messages
            .iter()
            .map(|x| serde_json::to_string_pretty(x).unwrap())
            .collect::<Vec<_>>();
        let messages = messages
            .into_iter()
            .reduce(|acc, elem| format!("{},{}", acc, elem))
            .unwrap();
        let format = format!("{{messages: [{}]}}", messages);
        f.write_str(&format)
    }
}

#[derive(Deserialize, Serialize)]
pub struct ApiMessage {
    pub role: String,
    pub content: String,
}

impl ApiMessage {
    pub fn to_message(&self) -> Message {
        let role = match self.role.as_str() {
            "user" => Entity::User,
            "system" => Entity::System,
            "assistant" => Entity::Assistant,
            _ => unreachable!(),
        };
        Message {
            role,
            content: self.content.clone(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Wrapper {
    pub conversation_id: String,
    pub message: Message,
}

pub struct AssistantClient {
    environment: Environment,
    client: Client,
    headers: HeaderMap,
}

impl AssistantClient {
    pub fn new() -> Self {
        let mut rtn = Self {
            environment: Environment::new(),
            client: Client::new(),
            headers: HeaderMap::new(),
        };
        rtn.setup_headers();
        rtn
    }

    pub async fn read_input(
        &self,
        mut stream: TcpStream,
        sender: Sender<(String, Message)>,
        mut receiver: Receiver<Arc<Mutex<Context>>>,
    ) {
        let mut buffer = [0; 50000];
        while let Ok(bytes) = stream.read(&mut buffer).await {
            if bytes == 0 {
                continue;
            }
            let buffer = &buffer[0..bytes];
            let message = serde_json::from_slice::<Wrapper>(buffer).unwrap();
            sender
                .send((message.conversation_id.clone(), message.message.clone()))
                .unwrap();
            let context = receiver.recv().await.unwrap();
            let messages = self.build_messages(&message, &context).await;
            let answer = self.make_request(&mut stream, &messages).await;
            context
                .lock()
                .await
                .new_message(message.conversation_id, answer);
        }
    }

    /*
     *
     */

    async fn make_request(&self, stream: &mut TcpStream, messages: &Messages) -> Message {
        let response = self
            .client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .headers(self.headers.clone())
            .json(&messages)
            .send()
            .await
            .unwrap();

        let response = response.json::<ApiResponse>().await.unwrap();
        stream
            .write(response.choices[0].message.content.as_bytes())
            .await
            .unwrap();
        response.choices[0].message.to_message()
    }

    fn setup_headers(&mut self) {
        let api_key = self.environment.get_key();
        self.headers.insert(
            AUTHORIZATION,
            format!("Bearer {}", api_key).parse().unwrap(),
        );
        self.headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    }

    async fn build_messages(&self, message: &Wrapper, context: &Arc<Mutex<Context>>) -> Messages {
        let messages = context
            .lock()
            .await
            .get_messages(&message.conversation_id)
            .iter()
            .map(|message| ApiMessage {
                role: message.role.to_string(),
                content: message.content.clone(),
            })
            .collect::<Vec<ApiMessage>>();

        let messages = Messages {
            messages,
            model: self.environment.get_model(),
        };
        messages
    }
}
