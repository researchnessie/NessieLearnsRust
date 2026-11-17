use crate::{bot_brain, storage_manager};
use colored::*;
use qrcode::QrCode;
use tokio::sync::Mutex;
use std::{collections::{HashMap, HashSet},time::Duration,sync::Arc};
use chrono::Local;
use whatsapp_rust::{prelude::*, download::MediaType, UploadOptions, media, media::ImageOptions};
//==================================================================================================
type RepliedCache = Arc<Mutex<HashMap<String, HashSet<bot_brain::MessageKey>>>>;
//==================================================================================================
pub async fn launch_bot() {
    let reply_bot = Arc::new(bot_brain::Brain::init());
    loop {
        match auto_bot(reply_bot.clone()).await {
            Ok(_) => {
                println!("Bot exited normally.");
            }
            Err(e) => {
                // Print the detailed error cause
                eprintln!("CRITICAL BOT ERROR: {:?}{:#?}", e, e);
            }
        }
        eprintln!("bot stopped, restarting in 10s...");
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
async fn auto_bot(reply_bot: Arc<bot_brain::Brain>) -> Result<(),Box<dyn std::error::Error>>{
    println!("{}", "Initializing WhatsApp Bot...".green());
    let replied_cache: RepliedCache = Arc::new(Mutex::new(HashMap::new()));
    let store = SqliteStore::new(storage_manager::login_data_storage().as_str()).await?;//wehre does this come from?
    let bot = Bot::builder().with_backend(store)
        .on_qr_code(|code, _timeout| async move {
            if let Ok(qr) = QrCode::new(code.as_bytes()) {
                let image = qr
                    .render::<qrcode::render::unicode::Dense1x2>()
                    .dark_color(qrcode::render::unicode::Dense1x2::Dark)
                    .light_color(qrcode::render::unicode::Dense1x2::Light)
                    .build();
                println!("\nScan this QR code with WhatsApp:\n{}\n", image);
            } else {
                println!("{}", "QR Code rendering failed!".red());
                println!("{}",format!("QR Code string: {code}").green());
            }
        }).on_message(move |ctx| {
            let cache = replied_cache.clone();
            let botty = reply_bot.clone();
            async move {process_message(ctx, cache, botty).await}
    }).build().await?;
    println!("{}", "Bot is running and listening for messages...".green());
    bot.run().await;
    Ok(())
}
async fn process_message(ctx:MessageContext, cache: RepliedCache, reply_bot:Arc<bot_brain::Brain>){
    if let Some(text) = ctx.message.text_content(){
        //on an event that a message arrives, we are separating and assigning variable to message, sender and receiver
        let message = text.to_lowercase();
        let mut sender = ctx.info.source.sender.to_string();
        if sender == "199007410372652:22@lid".to_string() {sender = String::from("Me")}
        let mut receiver = ctx.info.source.chat.to_string();
        if receiver == sender {receiver = String::from("Me")}
        //if sender != "80350717968563@lid"{return} //only for tests
        //==========================================================================================
        //this part adds all the senders to a file to be reviewed at the end of the day for missing, error or follow-ups
        if sender != "Me".to_string() {
            let mut users_to_be_appended = storage_manager::read_userid();
            let already_exists = users_to_be_appended.iter().any(|u|u.task == sender);
            if !already_exists {
                users_to_be_appended.push(storage_manager::Task{task: sender.clone(), due_time:Local::now(),done:false});
                storage_manager::write_userid(users_to_be_appended);
            }
        }
        //==========================================================================================
        //this part collects user messages to be trained by onnx, this code is temporary and will be removed when no longer needed
        let mut existing_messages = storage_manager::load_user_messages();
        existing_messages.push(message.clone());
        storage_manager::save_user_messages(existing_messages);
        //==========================================================================================
        //here we filter all the messages before feeding it to the brain
        if ctx.info.timestamp.timestamp() < reply_bot.start_time {return;} //ignore messages before launch
        if ctx.info.source.is_group {println!("{}",message.blue().on_white());return;} //return if it is group message
        if ctx.info.source.is_from_me {println!("{}",message.black().on_white());return;} //return if it is owner or bot
        if bot_brain::BLOCKED.iter().any(|b| b == &sender) {println!("{},{}", "[from blocked list]:".purple().on_white(),message.purple().on_white());return;} //return if it is from blocked list
        //after all the filtering if the message is still there, we will print it normally on the console to be read
        println!("{} -> {} : {}",sender,receiver,text); //text is used to preserve the original incoming message
        //==========================================================================================
        //now we can check for matching keys inside the brain
        let mut keys_to_reply = reply_bot.get_matching_keys(&message);
        //==========================================================================================
        //here we will check if the reply key we got is already replied to the customer
        let mut db = cache.lock().await;
        let user_replies = db.entry(sender.clone()).or_insert_with(HashSet::new);
        keys_to_reply.retain(|k| !user_replies.contains(k));
        if keys_to_reply.is_empty() {return;}//return if there are no keys remaining after cleaning up
        let key_clone = keys_to_reply.clone();
        //now we can finally ask the brain for replies on the remaining keys
        for k in keys_to_reply {
            if k != bot_brain::MessageKey::Acc { if sender != "80350717968563@lid" { user_replies.insert(k); }}
            if let Some(reply) = reply_bot.get_reply(k) {
                println!("{}",reply.magenta());
                let _ = ctx.reply(reply).await;
            }
        }
        //and now we need to check if the customer has asked for the rates before they are asking for account
        if key_clone.contains(&bot_brain::MessageKey::Acc) {
            if user_replies.contains(&bot_brain::MessageKey::AedMmk) || sender == "80350717968563@lid".to_string() {
                if sender!="80350717968563@lid".to_string(){user_replies.insert(bot_brain::MessageKey::Acc);}
                let bytes = reply_bot.aed_acc.clone();
                match ctx.client.upload(bytes, MediaType::Image, UploadOptions::default()).await{
                    Ok(upload) => {
                        let msg = media::image_message(
                            upload,
                            ImageOptions {
                                caption: Some(bot_brain::CAPTION.into()),
                                ..Default::default()
                            },
                        );
                        if let Err(e) = ctx.send_message(msg).await {
                            println!("Fail to send message: {}", e)
                        }
                    }
                    Err(e) => {
                        println!("Fail to upload: {}", e)
                    }
                }
            }
        }
    }
}
//==================================================================================================