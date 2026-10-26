mod bot_reply;
#[tokio::main]
async fn main() {
    bot_reply::launch_bot().await
}