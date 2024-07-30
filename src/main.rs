pub mod modules;

#[tokio::main]
async fn main() {
    let (mut server, listener, sender) = modules::input::Server::new().await;
    modules::input::receive_incoming(listener, sender).await;
    loop {
        server.serve().await;
    }
}
