//import crates
use dirs;
use colored::*;
use regex::Regex;
use qrcode::QrCode;
use walkdir::WalkDir;
use tokio::sync::Mutex;
use directories::ProjectDirs;
use console_utils::input::input;
use whatsapp_rust::{UploadOptions, download::MediaType, media, media::ImageOptions, prelude::*};
use std::{fs, sync::Arc, process::exit,collections::{HashMap, HashSet},time::{Duration, SystemTime, UNIX_EPOCH}};
//==================================================================================================
//general functions
fn store() -> String {
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
}//to store the log-in data
fn process_images(filename: &str) -> Option<Vec<u8>> {
    let download_folder = dirs::download_dir()?;
    for entry in WalkDir::new(download_folder).into_iter().flatten() {
        if entry.file_name() == filename && entry.file_type().is_file() {
            return fs::read(entry.path()).ok();
        }
    }
    None
} //search and return image paths
fn load_or_warn(filename: &str) -> Vec<u8> {
    match process_images(filename) {
        Some(data) => data,
        None => {
            println!(
                "{},{}",
                " is missing in the download folder!".red(),
                filename
            );
            exit(1)
        }
    }
} //convenience wrapper used at startup
fn is_glue(c: char) -> bool {
    c.is_ascii_alphanumeric()
}
fn has_keyword(text: &str, word: &str) -> bool {
    let guard_left = word.chars().next().is_some_and(is_glue);
    let guard_right = word.chars().next_back().is_some_and(is_glue);

    text.match_indices(word).any(|(start, found)| {
        let before = text[..start].chars().next_back(); // char just before the match
        let after = text[start + found.len()..].chars().next(); // char just after it
        let left_ok = !guard_left || !before.is_some_and(is_glue);
        let right_ok = !guard_right || !after.is_some_and(is_glue);
        left_ok && right_ok
    })
}
fn ask_aed_acc()->f64{
    loop{
        let x:String = input("AED_MMK Acc: ");
        match x.trim().parse::<f64>(){
            Ok(v) if v.is_finite() && v > 0.0 => return v,
            _ => {println!("{}","Invalid input, try again!".red())}
        }
    }
}
fn ask_aed_cash()->f64{
    loop{
        let x:String = input("AED_MMK Cash: ");
        match x.trim().parse::<f64>(){
            Ok(v) if v.is_finite() && v > 0.0 => return v,
            _ => {println!("{}","Invalid input, try again!".red())}
        }
    }
}
fn ask_mmk_aed()->f64{
    loop{
        let x:String = input("MMK_AED: ");
        match x.trim().parse::<f64>(){
            Ok(v) if v.is_finite() || v.is_sign_positive() => return v,
            _ => {println!("{}","Invalid input, try again!".red())}
        }
    }
}
//==================================================================================================
//main program
pub struct Botty{
    rates:[f64;3],
    start_time: i64,
    aed_acc: Vec<u8>,
    aed_reg:Regex,
    mmk_reg:Regex,
    one_aed_reg:Regex,
}
impl Botty{
    fn init()->Self{
        println!("{}", "Type your rates...".green());
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH).unwrap()
            .as_secs() as i64;
        Botty{
            rates:[ask_aed_acc(), ask_aed_cash(), ask_mmk_aed()],
            start_time:now,
            aed_acc:load_or_warn("aedpic.jpeg"),
            aed_reg:Regex::new(AED_REG).unwrap(),
            mmk_reg:Regex::new(MMK_REG).unwrap(),
            one_aed_reg:Regex::new(ONE_AED_REG).unwrap(),
        }
    }
    fn get_reply(&self, key:MessageKey) -> Option<String> {
        match key {
            MessageKey::Greeting => Some("မင်္ဂလာပါ".to_string()),
            MessageKey::Ticketing => {
                Some("Visa and Ticket နဲ့ပတ်သက်ပြီးသိလိုတာတွေကို Visa and Ticketing WhatsApp +971501238266 ကိုဆက်သွယ်မေးပေးပါ\nသူတို့ရဲ့ရုံးချိန် 9:00AM to 12:00AM(Midnight) အတွင်းမှာ ပြန်လည်ဖြေကြားပေးသွားပါလိမ့်မယ်".to_string())
            }
            MessageKey::Passport => {
                Some("Passport နဲ့ပတ်သက်ပြီးသိလိုတာတွေကို Office Reception WhatsApp +971501775318 ကိုဆက်သွယ်မေးပေးပါ\nသူတို့ရဲ့ရုံးချိန် 9:00AM to 3:00AM အတွင်းမှာ ပြန်လည်ဖြေကြားပေးသွားပါလိမ့်မယ်".to_string())
            }
            MessageKey::Food => {
                Some("မြန်မာအစားစာ နဲ့ပတ်သက်ပြီးသိလိုတာတွေကို TNT Food WhatsApp +971503490683 ကိုဆက်သွယ်မေးပေးပါ\nသူတို့ရဲ့ရုံးချိန် 9:00AM to 3:00AM အတွင်းမှာ ပြန်လည်ဖြေကြားပေးသွားပါလိမ့်မယ်".to_string())
            }
            MessageKey::OneAED => {
            Some(format!("မြန်မာပြည်ငွေလွှဲ 1 AED ကို\n{} MMK (Bank account transfer/Kpay/Wave/True Money)\n{} MMK (Cash out at Ygn Office/Cash Home Delivery/ ဘဏ်ထုတ်)\nရပါမယ် − ငွေစျေးပြောင်းနိုင်ပါတယ် - *ငွေပြန်လွှဲပေးတဲ့အခါမှာ 1AED နဲ့တွက်ပြီးမလွှဲပေးပါဘူးနော်*\nမှတ်ပုံတင်ဖြင့်လွှဲသည်ဖြစ်စေ နယ်အကောင့်များသို့ထည့်သည်ဖြစ်စေ ကျသင့်မည့် ဘဏ် charges များကို customer ဘက်မှသာကျခံပေးရပါမည်\nအိမ်ပို့ငွေများအားလုံး 4-5 ရက်အထိကြာနိုင်ပါတယ်\nငွေမလွှဲခင်တိုင်းအကောင့်ပြန် confirmပြီးမှလွှဲပေးပါနော်",
            (100000.0 / self.rates[0]).round() as i32,(100000.0 / self.rates[1]).round() as i32))
            },
            MessageKey::AedMmk => {
            Some(format!("မြန်မာပြည်ငွေလွှဲ 100,000 MMK ကို\n{} AED (Bank account transfer/Kpay/Wave/True Money)\n{} AED (Cash out at Ygn Office/Cash Home Delivery/ ဘဏ်ထုတ်)\nပေးရပါမယ် − ငွေစျေးပြောင်းနိုင်ပါတယ်\nမှတ်ပုံတင်ဖြင့်လွှဲသည်ဖြစ်စေ နယ်အကောင့်များသို့ထည့်သည်ဖြစ်စေ ကျသင့်မည့် ဘဏ် charges များကို customer ဘက်မှသာကျခံပေးရပါမည်\nအိမ်ပို့ငွေများအားလုံး 4-5 ရက်အထိကြာနိုင်ပါတယ်\nငွေမလွှဲခင်တိုင်းအကောင့်ပြန် confirmပြီးမှလွှဲပေးပါနော်",
            self.rates[0], self.rates[1]))
            },
            MessageKey::MmkAed => {
                if self.rates[2]!=0.0{
                    Some(format!("မြန်မာပြည်ကနေငွေလွဲမယ်ဆိုရင် တစ်သိန်းကို {} AED ရပါမယ် စျေးပြောင်းနိုင်ပါတယ်\nAED ချက်ချင်းပြန်လိုတယ်ဆိုရုံးကို လာထုတ်ပေးမှရပါမယ် အကောင့်ထဲကို လွှဲပေးရမယ်ဆို up to 48 hr အထိကြာနိုင်ပါတယ်",self.rates[2]))
                }
                else{
                    Some("လောလောဆယ် mmk to aed ငွေလွှဲပေးရန် stock မရှိသေးပါဘူး ငွေပြန်လွှဲလို့ရတဲ့အခါပြောပေးပါမယ်နော်".to_string())
                }
            },
            MessageKey::Acc => None
        }
    }
}
//==================================================================================================
pub async fn launch_bot() {
    let botty = Arc::new(Botty::init());
    loop {
        match auto_bot(botty.clone()).await {
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
pub async fn auto_bot(botty:Arc<Botty>) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Initializing WhatsApp Bot...".green());
    let replied_cache: RepliedCache = Arc::new(Mutex::new(HashMap::new()));
    let store = SqliteStore::new(store().as_str()).await?;
    let bot = Bot::builder()
        .with_backend(store)
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
        })
        .on_message(move |ctx| {
            let cache = replied_cache.clone();
            let botty = botty.clone();
            async move { process_message(ctx, cache, botty).await }
        })
        .build()
        .await?;
    println!("{}", "Bot is running and listening for messages...".green());
    bot.run().await;
    Ok(())
}
async fn process_message(ctx: MessageContext, cache: RepliedCache, botty:Arc<Botty>) {
    if ctx.info.timestamp.timestamp() < botty.start_time {return;} //ignore messages before launch
    if let Some(text) = ctx.message.text_content() {
        let message = text.to_lowercase();
        let mut sender = ctx.info.source.sender.to_string();
        if sender == "199007410372652:22@lid".to_string() {sender = String::from("Me")}
        let mut receiver = ctx.info.source.chat.to_string();
        if receiver == sender {receiver = String::from("Me")}
        //if sender != "80350717968563@lid"{return} //only for tests
        //==========================================================================================
        if ctx.info.source.is_group {println!("{}",message.blue().on_white());return;} //return if it is group message
        if ctx.info.source.is_from_me {println!("{}",message.black().on_white());return;} //return if it is owner or bot
        if BLOCKED.iter().any(|b| b == &sender) {println!("{},{}", "[from blocked list]:".purple().on_white(),message.purple().on_white());return;} //return if it is from blocked list
        println!("Sender: {} -> Receiver: {} = {}",
                 sender.to_string(),
                 receiver.to_string(),
                 text
        );
        //==========================================================================================
        let mut key = vec![];

        if GREETING.iter().any(|&word| has_keyword(&message,word)) {key.push(MessageKey::Greeting)}
        if TICKETING.iter().any(|&word| has_keyword(&message, word)) {key.push(MessageKey::Ticketing)}
        if PASSPORT.iter().any(|&word| has_keyword(&message,word)) {key.push(MessageKey::Passport)}
        if FOOD.iter().any(|&word| has_keyword(&message,word)) {key.push(MessageKey::Food)}
        if botty.aed_reg.is_match(&message){key.push(MessageKey::AedMmk)}
        if botty.mmk_reg.is_match(&message){key.push(MessageKey::MmkAed)}
        if botty.one_aed_reg.is_match(&message){key.push(MessageKey::OneAED)}
        if ACCOUNT.iter().any(|&word| has_keyword(&message,word)) {key.push(MessageKey::Acc)}
        if key.is_empty() {return;} //return if there are no reply available

        //double reply checker
        let mut db = cache.lock().await;
        let user_replies = db.entry(sender.clone()).or_insert_with(HashSet::new);
        key.retain(|k| !user_replies.contains(k));
        let key_clone = key.clone();
        if key.is_empty() {return;}
        for k in key {
            if k != MessageKey::Acc { if sender != "80350717968563@lid" { user_replies.insert(k); }}
            if let Some(reply) = botty.get_reply(k) {
                println!("{}",reply.magenta());
                let _ = ctx.reply(reply).await;
            }
        }
        if key_clone.contains(&MessageKey::Acc) {
            if user_replies.contains(&MessageKey::OneAED) || user_replies.contains(&MessageKey::AedMmk) || sender == "80350717968563@lid".to_string() {
                if sender!="80350717968563@lid".to_string(){user_replies.insert(MessageKey::Acc);}
                let bytes = botty.aed_acc.clone();
                match ctx.client.upload(bytes, MediaType::Image, UploadOptions::default()).await{
                    Ok(upload) => {
                        let msg = media::image_message(
                            upload,
                            ImageOptions {
                                caption: Some(CAPTION.into()),
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
//types needed
type RepliedCache = Arc<Mutex<HashMap<String, HashSet<MessageKey>>>>;
//==================================================================================================
//enums needed
#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
enum MessageKey{
    Greeting,Ticketing,Passport,Food,OneAED,AedMmk,MmkAed,Acc
}
//==================================================================================================
//constants
const BLOCKED: [&str; 16] = {[
    "142490036342981@lid", //Khin La Pyae
    "216509955743928@lid", //OmanAgent - AyeThandarHaling
    "224953794322458@lid", //OmanAgent - SeineLaeLae
    "97771004662011@lid",  //OmanAgent - Lydia
    "45754437627913@lid",  //KoWaiYan Qatar
    "146437111312478@lid", //TNT Ticketing
    "222075646177373@lid", //TNT Reception
    "143576679846075@lid", //TNT Food
    "73676020048049@lid",  //Thazin Tint
    "37392203395228@lid",  //Mr.TNT
    "138418692587692@lid", //Shumawa
    "40252852945107@lid",  //AKS
    "135093918113861@lid", //KZL
    "17867584094214@lid",  //Phyo MaungMaung
    "200992322777123@lid", //TTW
    "102723068424257@lid", //KTDL Oman
    //"199007410372652:22@lid" //ONLINE WA
    //"80350717968563@lid", //Me
]};
const ACCOUNT: [&str; 3] = ["acc", "account", "အကောင့်"];
const GREETING: [&str; 7] = ["hi", "hello", "မင်္ဂလာ", "မဂလာ", "မဂ်လာ", "xi", "ဟိုင်း"];
const TICKETING: [&str; 5] = ["ticketing", "ticket", "visa", "လက်မှတ်", "ဗီဇာ"];
const PASSPORT: [&str; 3] = ["passport", "ပတ်စပို့", "သက်တမ်းတိုး"];
const FOOD: [&str; 3] = ["food", "စားစရာ", "မုန့်"];
const ONE_AED_REG:&str = r"(?i)(?-u:\b)1\s*(-)?\s*(aed|ကျပ်)";
const AED_REG:&str = r"(?i)(?-u:\b)(aed|dubai|uae|ဒူဘိုင်း|rate|စျေး|ငွေ)\s*(to)?\s*(ကနေ)?\s*(က)?\s*(-)?\s*(mmk|mk|kyat|myanmar|kpay|k pay|wave|မြန်မာ|rate|စျေး)";
const MMK_REG:&str = r"(?i)(?-u:\b)(mmk|mk|kyat|myanmar|kpay|k pay|wave|မြန်မာ)\s*(to)?\s*(ကနေ)?\s*(က)?\s*(-)?\s*(aed|dubai|uae|ဒူဘိုင်း)";
const CAPTION: &str =   "ပုံထဲမှာ ရေးထားတာလေးသေချာသတိထားပေးပါ ကျေးဇူးတင်ပါတယ်\nလွှဲထားတဲ့စလစ်လေးကို နေ့တိုင်းမနက် 6:00AM မှာငွေစစ်ပြီးတော့ ငွေဝင်ထားတဲ့နေ့မှာမြန်မာပြည်ကနေပြန်လွှဲပေးပါတယ်";