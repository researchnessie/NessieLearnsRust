mod auto_reply;
mod test;
#[tokio::main]
async fn main() {
    auto_reply::launch_bot().await
}