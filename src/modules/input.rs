use crate::modules::client::AssistantClient;
use crate::modules::context::{Context, Message};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc::{
    unbounded_channel, UnboundedReceiver as Receiver, UnboundedSender as Sender,
};
use tokio::sync::Mutex;

type ContextSender = HashMap<String, Sender<Arc<Mutex<Context>>>>;
type MessageReceiver = HashMap<String, Receiver<(String, Message)>>;

pub struct Server {
    contexts: Arc<Mutex<Context>>,
    receivers: MessageReceiver,
    senders: ContextSender,
    channel_transfer: Receiver<(
        String,
        Sender<Arc<Mutex<Context>>>,
        Receiver<(String, Message)>,
    )>,
}

impl Server {
    pub async fn new() -> (
        Self,
        TcpListener,
        Sender<(
            String,
            Sender<Arc<Mutex<Context>>>,
            Receiver<(String, Message)>,
        )>,
    ) {
        let (sender, receiver) = unbounded_channel();
        let listener = TcpListener::bind("127.0.0.1:2469").await.unwrap();
        (
            Self {
                contexts: Arc::new(Mutex::new(Context::new().await)),
                receivers: HashMap::new(),
                senders: HashMap::new(),
                channel_transfer: receiver,
            },
            listener,
            sender,
        )
    }

    pub async fn serve(&mut self) {
        while !self.channel_transfer.is_empty() {
            let (id, sender, receiver) = self.channel_transfer.recv().await.unwrap();
            self.senders.insert(id.clone(), sender);
            self.receivers.insert(id.clone(), receiver);
        }
        for (id, receiver) in self.receivers.iter_mut() {
            let sender = self.senders.get(id).unwrap();
            while !receiver.is_empty() {
                println!("processing message from {}", id);
                let (conversation_id, message) = receiver.recv().await.unwrap();
                let mut lock = self.contexts.lock().await;
                lock.new_message(conversation_id, message);
                sender.send(self.contexts.clone()).unwrap();
            }
        }
    }
}

pub async fn receive_incoming(
    listener: TcpListener,
    sender: Sender<(
        String,
        Sender<Arc<Mutex<Context>>>,
        Receiver<(String, Message)>,
    )>,
) {
    tokio::spawn(async move {
        while let Ok((stream, addr)) = listener.accept().await {
            let addr = addr.to_string();
            let (context_sender, context_receiver) = tokio::sync::mpsc::unbounded_channel();
            let (message_sender, message_receiver) = tokio::sync::mpsc::unbounded_channel();
            sender
                .send((addr, context_sender, message_receiver))
                .unwrap();
            let client = AssistantClient::new();
            tokio::spawn(async move {
                client
                    .read_input(stream, message_sender, context_receiver)
                    .await;
            });
        }
    });
}
