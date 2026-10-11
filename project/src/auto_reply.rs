//imports
use dirs;
use qrcode::QrCode;
use walkdir::WalkDir;
use directories::ProjectDirs;
use tokio::sync::Mutex;
use whatsapp_rust::prelude::*;
use console_utils::input::{input};
use std::{fs, collections::{HashMap,HashSet}, sync::{Arc,OnceLock},thread, time::{Duration, SystemTime, UNIX_EPOCH}, process::exit};
use whatsapp_rust::download::MediaType;
use whatsapp_rust::{media, UploadOptions};
use whatsapp_rust::media::{ImageOptions};
//==================================================================================================
//modify terminal display by replacing sending messages as "ME" on user
//
//implement auto account sending
//add gui indicator for being live and exit button
//==================================================================================================
//general functions
fn wait(seconds:f64){thread::sleep(Duration::from_secs(seconds as u64));}//simplifying wait for multiple uses
fn store()->String{
    let db_path = if let Some(proj_dirs) = ProjectDirs::from("com", "AutoReply", "WhatsAppBot") {
        let data_dir = proj_dirs.data_dir();

        // Automatically creates the folder (e.g., AppData/Roaming/YourOrganization/WhatsAppBot) if it doesn't exist
        fs::create_dir_all(data_dir).expect("Failed to create app data directory!");

        data_dir.join("whatsapp_session.db")
    } else {
        // Fallback if OS doesn't support project dirs
        std::path::PathBuf::from("whatsapp_session.db")
    };
    db_path.to_str().unwrap().to_string()
}
fn get_rates(){
    println!("Type your rates...");
    wait(0.5);
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
    } else{let _ = RATES.set(ready_rates);}
}//get initial rates
fn process_images(filename:&str)->Option<Vec<u8>>{
    let download_folder = dirs::download_dir()?;
    for entry in WalkDir::new(download_folder).into_iter().flatten(){
        if entry.file_name() == filename && entry.file_type().is_file() {
            return fs::read(entry.path()).ok();
        }
    } None
}//search and decode images into bytes
fn load_or_warn(filename: &str) -> Vec<u8> {
    match process_images(filename) {
        Some(data) => data,
        None => {
            println!("{} is missing in the download folder!", filename);
            exit(0)
        },
    }
}//convenience wrapper used at startup
//==================================================================================================
pub async fn launch_bot() {
    //load_or_warn("aedpic.jpeg");
    get_rates();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let _ = START_TIME.set(now - 5);
    let _ = AED_ACC.set(load_or_warn("aedpic.jpeg"));
    auto_bot().await.unwrap();
}
pub async fn auto_bot() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing WhatsApp Bot...");
    let replied_cache :RepliedCache = Arc::new(Mutex::new(HashMap::new()));
    let store = SqliteStore::new(store().as_str()).await?;
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
        .on_message(move |ctx| {
            let cache = replied_cache.clone();
            async move {
                process_message(ctx, cache).await
            }
        })
        .build()
        .await?;

    println!("Bot is running and listening for messages...");
    bot.run().await;
    Ok(())
}
async fn process_message(ctx: MessageContext, cache: RepliedCache) {
    if ctx.info.timestamp.timestamp() < *START_TIME.get().unwrap() {return;} //ignore messages before launch

    if let Some(text) = ctx.message.text_content() {
        let message = text.to_lowercase();
        let mut sender = ctx.info.source.sender.to_string();
        if sender == String::from("199007410372652:22@lid"){sender = String::from("Me")}
        let mut receiver = ctx.info.source.chat.to_string();
        if receiver == String::from("199007410372652:22@lid"){receiver = String::from("Me")}
        println!("Sender: {} -> Receiver: {} = {}",sender.to_string(),receiver.to_string(),text);
        //==========================================================================================
        if ctx.info.source.is_group{return;} //return if it is group message
        if ctx.info.source.is_from_me{return;} //return if it is owner or bot
        if BLOCKED.iter().any(|b| b==&sender){println!("from blocked list");return;} //return if it is from blocked list
        //==========================================================================================
        let mut key ="NAR";

             if ACCOUNT.iter().any(|&word| message.contains(word)){key="acc"}
        else if TICKETING.iter().any(|&word| message.contains(word)){key="ticketing"}
        else if PASSPORT.iter().any(|&word| message.contains(word)){key="passport"}
        else if FOOD.iter().any(|&word| message.contains(word)){key="food"}
        else if ONE_AED.iter().any(|&word| message.contains(word)){key="one_aed"}
        else if AED_MMK_RATE.iter().any(|&word| message.contains(word)){key="aed_mmk"}
        else if MMK_AED_RATE.iter().any(|&word| message.contains(word)){key="mmk_aed"}
        else if GREETING.iter().any(|&word| message.contains(word)) {key="greeting"}
        else if key=="NAR"{return} //return if there are no reply available
        //double reply checker
        let mut db = cache.lock().await;
        let user_replies = db.entry(sender.clone()).or_insert_with(HashSet::new);
        if user_replies.contains(key){println!("Skipped duplicate reply for {} -> key = {}", sender.to_string(),key);}
        else if key == "acc"{
            if user_replies.contains("aed_mmk") || user_replies.contains("one_aed"){
                user_replies.insert(String::from(key));
                let bytes = AED_ACC.get().unwrap().clone();
                match ctx.client.upload(bytes, MediaType::Image, UploadOptions::default()).await{
                    Ok(upload) => {
                        let msg = media::image_message(upload,
                        ImageOptions{caption:Some("Caption!".into()), ..Default::default()},);
                        if let Err(e) = ctx.send_message(msg).await{
                            println!("Fail to send message: {}", e)
                        }
                    }
                    Err(e) => {println!("Fail to upload: {}", e)}

                }
            }

        }

        else if get_reply(key.to_string()) != "No available reply"{
            user_replies.insert(key.to_string());
            let _ = ctx.reply(get_reply(key.to_string())).await;}
    }
}
//==================================================================================================
//Reply functions
fn get_reply(key:String)->String{
    let rates = RATES.get().unwrap();
    let reply = match key.as_str(){
        "greeting"=>"မင်္ဂလာပါ",
        "ticketing"=>"Visa and Ticket နဲ့ပတ်သက်ပြီးသိလိုတာတွေကို Visa and Ticketing WhatsApp +971501238266 ကိုဆက်သွယ်မေးပေးပါ\nသူတို့ရဲ့ရုံးချိန် 9:00AM to 12:00AM(Midnight) အတွင်းမှာ ပြန်လည်ဖြေကြားပေးသွားပါလိမ့်မယ်",
        "office"=>"TNT Office မှာငွေလွှဲထားတယ်ဆိုရင် Office Reception WhatsApp +971501775318 ကိုဆက်သွယ်မေးပေးပါ\nသူတို့ရဲ့ရုံးချိန် 9:00AM to 3:00AM အတွင်းမှာ ပြန်လည်ဖြေကြားပေးသွားပါလိမ့်မယ်",
        "passport"=>"Passport နဲ့ပတ်သက်ပြီးသိလိုတာတွေကို Office Reception WhatsApp +971501775318 ကိုဆက်သွယ်မေးပေးပါ\nသူတို့ရဲ့ရုံးချိန် 9:00AM to 3:00AM အတွင်းမှာ ပြန်လည်ဖြေကြားပေးသွားပါလိမ့်မယ်",
        "food"=>"မြန်မာအစားစာ နဲ့ပတ်သက်ပြီးသိလိုတာတွေကို TNT Food WhatsApp +971503490683 ကိုဆက်သွယ်မေးပေးပါ\nသူတို့ရဲ့ရုံးချိန် 9:00AM to 3:00AM အတွင်းမှာ ပြန်လည်ဖြေကြားပေးသွားပါလိမ့်မယ်",
        "one_aed"=>&format!(
"မြန်မာပြည်ငွေလွှဲ 1 AED ကို
{} MMK (Bank account transfer/Kpay/Wave/True Money)
{} MMK (Cash out at Ygn Office/Cash Home Delivery/ ဘဏ်ထုတ်)
ရပါမယ် − ငွေစျေးပြောင်းနိုင်ပါတယ် - *ငွေပြန်လွှဲပေးတဲ့အခါမှာ 1AED နဲ့တွက်ပြီးမလွှဲပေးပါဘူးနော်*
မှတ်ပုံတင်ဖြင့်လွှဲသည်ဖြစ်စေ နယ်အကောင့်များသို့ထည့်သည်ဖြစ်စေ ကျသင့်မည့် ဘဏ် charges များကို customer ဘက်မှသာကျခံပေးရပါမည်
အိမ်ပို့ငွေများအားလုံး 4-5 ရက်အထိကြာနိုင်ပါတယ်
ငွေမလွှဲခင်တိုင်းအကောင့်ပြန် confirmပြီးမှလွှဲပေးပါနော်",
(100000.0/rates[0]).round() as i32, (100000.0/rates[1]).round() as i32),
        "aed_mmk"=>&format!(
"မြန်မာပြည်ငွေလွှဲ 100,000 MMK ကို
{} AED (Bank account transfer/Kpay/Wave/True Money)
{} AED (Cash out at Ygn Office/Cash Home Delivery/ ဘဏ်ထုတ်)
ပေးရပါမယ် − ငွေစျေးပြောင်းနိုင်ပါတယ်
မှတ်ပုံတင်ဖြင့်လွှဲသည်ဖြစ်စေ နယ်အကောင့်များသို့ထည့်သည်ဖြစ်စေ ကျသင့်မည့် ဘဏ် charges များကို customer ဘက်မှသာကျခံပေးရပါမည်
အိမ်ပို့ငွေများအားလုံး 4-5 ရက်အထိကြာနိုင်ပါတယ်
ငွေမလွှဲခင်တိုင်းအကောင့်ပြန် confirmပြီးမှလွှဲပေးပါနော်",
rates[0],rates[1]),
        "mmk_aed"=>&format!(
"မြန်မာပြည်ကနေငွေလွဲမယ်ဆိုရင် တစ်သိန်းကို {} AED ရပါမယ် စျေးပြောင်းနိုင်ပါတယ်
AED ချက်ချင်းပြန်လိုတယ်ဆိုရုံးကို လာထုတ်ပေးမှရပါမယ် အကောင့်ထဲကို လွှဲပေးရမယ်ဆို up to 48 hr အထိကြာနိုင်ပါတယ်",
rates[2]),
        _ => "No available reply"
    };
    reply.to_string()
}
//==================================================================================================
//types
type RepliedCache = Arc<Mutex<HashMap<String, HashSet<String>>>>;
//constants
const BLOCKED:[&str;16] = [
    "142490036342981@lid",  //Khin La Pyae
    "216509955743928@lid",  //OmanAgent - AyeThandarHaling
    "224953794322458@lid",  //OmanAgent - SeineLaeLae
    "97771004662011@lid",   //OmanAgent - Lydia
    "45754437627913@lid",   //KoWaiYan Qatar
    "146437111312478@lid",  //TNT Ticketing
    "222075646177373@lid",  //TNT Reception
    "143576679846075@lid",  //TNT Food
    "73676020048049@lid",   //Thazin Tint
    "37392203395228@lid",   //Mr.TNT
    "138418692587692@lid",  //Shumawa
    "40252852945107@lid",   //AKS
    "135093918113861",      //KZL
    "17867584094214@lid",   //Phyo MaungMaung
    "200992322777123@lid",  //TTW
    "102723068424257@lid",   //KTDL Oman
    //"199007410372652:22@lid" //ONLINE WA
    //"80350717968563@lid", //Me
];
const ACCOUNT:[&str;3] = ["acc","account", "အကောင့်"];
const GREETING:[&str;7]=["hi", "hello", "မင်္ဂလာ", "မဂလာ", "မဂ်လာ", "xi", "ဟိုင်း"];
const TICKETING: [&str;5] = ["ticketing", "ticket", "visa", "လက်မှတ်", "ဗီဇာ"];
const PASSPORT: [&str;3] = ["passport", "ပတ်စပို့", "သက်တမ်းတိုး"];
const FOOD: [&str;3] = ["food", "စားစရာ", "မုန့်"];
const ONE_AED:[&str;4] = ["1aed","1 ကျပ်", "1 aed", "1ကျပ်"];
const AED_MMK_RATE:[&str;22] = [
    "aed to mmk", "aedtommk", "aed tommk", "aedto mmk",
    "aed to mk", "aedtomk", "aed tomk", "aedto mk",
    "aed mmk", "aed mk", "aedmmk", "aedmk",
    "aed to myanmar", "aedto myanmar", "aed tomyanmar", "aedtomyanmar",
    "aed to kpay", "aedto kpay", "aed tokpay", "aedtokpay",
    "မြန်မာပြည်ကို", "မြန်မာကိုလွှဲ"
];
const MMK_AED_RATE:[&str;10] = [
    "mmk to aed", "mmk aed", "mmk→aed", "mk to aed", "mk aed", "ဒီဘက်မှာAED", "မြန်မာကနေ",
    "မြန်မာပြည်ကနေ", "kpay to aed", "မြန်မာငွေပေးပီး aed ယူ"
];

static RATES :OnceLock<Vec<f64>> = OnceLock::new();
static START_TIME: OnceLock<i64> = OnceLock::new();
static AED_ACC:OnceLock<Vec<u8>> = OnceLock::new();