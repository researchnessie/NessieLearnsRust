mod listener;
//mod spotify_handler;

#[tokio::main]
async fn main() {
    listener::listen().await;
    //let _ = spotify_handler::handler().await;
}