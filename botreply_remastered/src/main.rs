mod run;
mod storage_manager;
mod bot_brain;
#[tokio::main]
async fn main() {run::launch_bot().await;}
