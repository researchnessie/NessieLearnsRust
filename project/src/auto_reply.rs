//imports
use dirs;
use image;
use qrcode::QrCode;
use arboard::ImageData;
use std::process::exit;
use walkdir::{WalkDir};
use whatsapp_rust::prelude::*;
use console_utils::input::{input};
use std::{thread, time::Duration};
//==================================================================================================
//add dynamic user lists to keep tracked of already replied keywords
//and
//add a gate for trigger keys to prevent duplicate replies

//make a bunch of lists/dictionaries to add replies

//trigger all lists replies or else go to dynamic replies

//make dynamic replies with functions

//implement rate input on lunch
//implement auto account sending
//add gui indicator for being live and exit button
//==================================================================================================
//general functions
fn get_rates()->Vec<f64>{
    let aed_mmk_acc:String = input("AED_MMK Acc: ");
    let aed_mmk_cash:String = input("AED_MMK Cash: ");
    let mmk_aed:String = input("MMK_AED: ");
    let check_rates = [aed_mmk_acc, aed_mmk_cash, mmk_aed];
    let mut ready_rates = vec![];
    for r in check_rates.iter(){
        match r.trim().parse::<f64>(){
            Ok(val) => ready_rates.push(val),
            Err(_)=> ready_rates.push(0.0),
        }
    }
    if ready_rates.contains(&0.0){
        println!("WARNING: Rates Error!");
        get_rates()
    } else{ready_rates}
}//get initial rates
fn wait(seconds:f64){
    thread::sleep(Duration::from_secs(seconds as u64));
}//simplifying wait for multiple uses
fn process_images(filename:&str)->Option<ImageData<'static>>{
    let download_folder = dirs::download_dir()?;
    for entry in WalkDir::new(download_folder).into_iter(){
        let Ok(entry) = entry else {continue};
        if entry.file_name() == filename && entry.file_type().is_file() {
            let picture = image::open(entry.path()).ok()?.to_rgba8();
            return Some(ImageData {
                width:picture.width() as usize,
                height:picture.height() as usize,
                bytes:picture.into_raw().into(),
            })
        }
    } None
}//search and decode images into bytes
fn load_or_warn(filename: &str) -> ImageData<'static> {
    match process_images(filename) {
        Some(data) => data,
        None=> {println!("{}, is missing or corrupted.Please add the file in current user's download folder and relaunch the program!",filename);exit(0);}
    }
}//convenience wrapper used at startup
//==================================================================================================
pub async fn launch_bot() {
    //load_or_warn("aedpic.jpeg");
    //let rates = get_rates();
    //println!("rate: {:?}", rates);
    auto_bot().await.unwrap();
}
pub async fn auto_bot() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing WhatsApp Bot...");
    let store = SqliteStore::new("whatsapp_session.db").await?;
    let bot = Bot::builder().with_backend(store)
        .on_qr_code(|code, _timeout| async move {
            if let Ok(qr) = QrCode::new(code.as_bytes()){
                let image = qr.render::<qrcode::render::unicode::Dense1x2>()
                    .dark_color(qrcode::render::unicode::Dense1x2::Dark)
                    .light_color(qrcode::render::unicode::Dense1x2::Light)
                    .build();
                println!("\nScan this QR code with WhatsApp:\n{}\n", image);
            }else{
                println!("QR Code rendering filed!");
                println!("QR Code string: {code}");
            }
        })
        .on_message(|ctx| async move {process_message(ctx).await})
        .build()
        .await?;

    println!("Bot is running and listening for messages...");
    bot.run().await;
    Ok(())
}
async fn process_message(ctx: MessageContext) {
    if let Some(text) = ctx.message.text_content() {
        let sender = ctx.info.source.sender.to_string();
        let message = text.to_lowercase();
        println!("{} -> {}",sender.to_string(),text);
        if reroute(&message)!="NAR"{let _ =ctx.reply(reroute(&message));}
    }
}
//==================================================================================================
//Reply functions

//rate related
fn aed_to_mmk(){}
fn one_aed_mmk(){}
fn mmk_to_aed(){}
//reroute related
fn reroute(where_to:&String)->&'static str{
    let reply = match where_to.as_str(){
        "ticketing"=>"Visa and Ticket နဲ့ပတ်သက်ပြီးသိလိုတာတွေကို Visa and Ticketing WhatsApp +971501238266 ကိုဆက်သွယ်မေးပေးပါ\nသူတို့ရဲ့ရုံးချိန် 9:00AM to 12:00AM(Midnight) အတွင်းမှာ ပြန်လည်ဖြေကြားပေးသွားပါလိမ့်မယ်",
        "office"=>"TNT Office မှာငွေလွှဲထားတယ်ဆိုရင် Office Reception WhatsApp +971501775318 ကိုဆက်သွယ်မေးပေးပါ\nသူတို့ရဲ့ရုံးချိန် 9:00AM to 3:00AM အတွင်းမှာ ပြန်လည်ဖြေကြားပေးသွားပါလိမ့်မယ်",
        "passport"=>"Passport နဲ့ပတ်သက်ပြီးသိလိုတာတွေကို Office Reception WhatsApp +971501775318 ကိုဆက်သွယ်မေးပေးပါ\nသူတို့ရဲ့ရုံးချိန် 9:00AM to 3:00AM အတွင်းမှာ ပြန်လည်ဖြေကြားပေးသွားပါလိမ့်မယ်",
        "food"=>"မြန်မာအစားစာ နဲ့ပတ်သက်ပြီးသိလိုတာတွေကို TNT Food WhatsApp +971503490683 ကိုဆက်သွယ်မေးပေးပါ\nသူတို့ရဲ့ရုံးချိန် 9:00AM to 3:00AM အတွင်းမှာ ပြန်လည်ဖြေကြားပေးသွားပါလိမ့်မယ်",
        _ => "NAR"
    };
    reply
}
//==================================================================================================
