//imports
use colored::*;
use console_utils::input::input;
use directories::ProjectDirs;
use dirs;
use qrcode::QrCode;
use std::{
    backtrace,
    collections::{HashMap, HashSet},
    fs,
    io::Write,
    panic,
    process::exit,
    sync::{Arc, OnceLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex;
use walkdir::WalkDir;
use whatsapp_rust::{UploadOptions, download::MediaType, media, media::ImageOptions, prelude::*};
//==================================================================================================
//implement auto account sending
//fixes
//customer keyword garbage (this = hi)
//get KZL id and block it
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
}
fn get_rates() {
    println!("{}", "Type your rates...".purple());
    let aed_mmk_acc: String = input("AED_MMK Acc: ");
    let aed_mmk_cash: String = input("AED_MMK Cash: ");
    let mmk_aed: String = input("MMK_AED: ");
    let check_rates = [aed_mmk_acc, aed_mmk_cash, mmk_aed];
    let mut ready_rates = vec![];
    for r in check_rates.iter() {
        match r.trim().parse::<f64>() {
            Ok(val) => ready_rates.push(val),
            Err(_) => ready_rates.push(0.0),
        }
    }
    if ready_rates.contains(&0.0) {
        println!("{}", "WARNING: Rates Error!".red());
        get_rates()
    } else {
        let _ = RATES.set(ready_rates);
    }
} //get initial rates
fn process_images(filename: &str) -> Option<Vec<u8>> {
    let download_folder = dirs::download_dir()?;
    for entry in WalkDir::new(download_folder).into_iter().flatten() {
        if entry.file_name() == filename && entry.file_type().is_file() {
            return fs::read(entry.path()).ok();
        }
    }
    None
} //search and decode images into bytes
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
//==================================================================================================
pub async fn launch_bot() {
    panic::set_hook(Box::new(|info| {
        let msg = format!("{info}\n{}\n", backtrace::Backtrace::force_capture());
        eprintln!("{}", msg);
        if let Ok(mut f) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("crash.log")
        {
            let _ = writeln!(f, "{msg}");
        }
    }));
    get_rates();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let _ = START_TIME.set(now - 5);
    let _ = AED_ACC.set(load_or_warn("aedpic.jpeg"));
    loop {
        if let Err(e) = auto_bot().await {
            eprintln!("bot error: {e}");
        }
        eprintln!("bot stopped, restarting in 10s...");
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
pub async fn auto_bot() -> Result<(), Box<dyn std::error::Error>> {
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
                println!("{}", "QR Code rendering filed!".red());
                println!("{}", "QR Code string: {code}".purple());
            }
        })
        .on_message(move |ctx| {
            let cache = replied_cache.clone();
            async move { process_message(ctx, cache).await }
        })
        .build()
        .await?;

    println!("{}", "Bot is running and listening for messages...".green());
    bot.run().await;
    Ok(())
}
async fn process_message(ctx: MessageContext, cache: RepliedCache) {
    if ctx.info.timestamp.timestamp() < *START_TIME.get().unwrap() {
        return;
    } //ignore messages before launch

    if let Some(text) = ctx.message.text_content() {
        let message = text.to_lowercase();
        let mut sender = ctx.info.source.sender.to_string();

        //if sender != "80350717968563@lid"{return} //only for tests

        if sender == String::from("199007410372652:22@lid") {
            sender = String::from("Me")
        }
        let mut receiver = ctx.info.source.chat.to_string();
        if receiver == String::from("199007410372652:22@lid") {
            receiver = String::from("Me")
        }
        println!(
            "Sender: {} -> Receiver: {} = {}",
            sender.to_string(),
            receiver.to_string(),
            text
        );
        //==========================================================================================
        if ctx.info.source.is_group {
            return;
        } //return if it is group message
        if ctx.info.source.is_from_me {
            return;
        } //return if it is owner or bot
        if BLOCKED.iter().any(|b| b == &sender) {
            println!("{}", "from blocked list".purple());
            return;
        } //return if it is from blocked list
        
        //==========================================================================================
        let mut key = vec![];

        if GREETING.iter().any(|&word| message.contains(word)) {
            key.push("greeting")
        }
        if TICKETING.iter().any(|&word| message.contains(word)) {
            key.push("ticketing")
        }
        if PASSPORT.iter().any(|&word| message.contains(word)) {
            key.push("passport")
        }
        if FOOD.iter().any(|&word| message.contains(word)) {
            key.push("food")
        }
        if ONE_AED.iter().any(|&word| message.contains(word)) {
            key.push("one_aed")
        }
        if AED_MMK_RATE.iter().any(|&word| message.contains(word)) {
            key.push("aed_mmk")
        }
        if MMK_AED_RATE.iter().any(|&word| message.contains(word)) {
            key.push("mmk_aed")
        }
        if ACCOUNT.iter().any(|&word| message.contains(word)) {
            key.push("acc")
        }
        if key.is_empty() {
            return;
        } //return if there are no reply available
        //double reply checker
        let mut db = cache.lock().await;
        let user_replies = db.entry(sender.clone()).or_insert_with(HashSet::new);
        key.retain(|k| !user_replies.contains(*k));
        let key_clone = key.clone();
        if key.is_empty() {
            return;
        }
        for k in key {
            if k != "acc" {
                user_replies.insert(k.to_string());
            }
            let reply = get_reply(k.to_string());
            if reply != "No available reply" {
                let _ = ctx.reply(reply).await;
            }
        }
        if key_clone.contains(&"acc") {
            if user_replies.contains("aed_mmk") || user_replies.contains("one_aed") {
                user_replies.insert(String::from("acc"));
                let bytes = AED_ACC.get().unwrap().clone();
                match ctx
                    .client
                    .upload(bytes, MediaType::Image, UploadOptions::default())
                    .await
                {
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
//Reply functions
fn get_reply(key: String) -> String {
    let rates = RATES.get().unwrap();
    let reply = match key.as_str() {
        "greeting" => "မင်္ဂလာပါ",
        "ticketing" => {
            "Visa and Ticket နဲ့ပတ်သက်ပြီးသိလိုတာတွေကို Visa and Ticketing WhatsApp +971501238266 ကိုဆက်သွယ်မေးပေးပါ\nသူတို့ရဲ့ရုံးချိန် 9:00AM to 12:00AM(Midnight) အတွင်းမှာ ပြန်လည်ဖြေကြားပေးသွားပါလိမ့်မယ်"
        }
        "passport" => {
            "Passport နဲ့ပတ်သက်ပြီးသိလိုတာတွေကို Office Reception WhatsApp +971501775318 ကိုဆက်သွယ်မေးပေးပါ\nသူတို့ရဲ့ရုံးချိန် 9:00AM to 3:00AM အတွင်းမှာ ပြန်လည်ဖြေကြားပေးသွားပါလိမ့်မယ်"
        }
        "food" => {
            "မြန်မာအစားစာ နဲ့ပတ်သက်ပြီးသိလိုတာတွေကို TNT Food WhatsApp +971503490683 ကိုဆက်သွယ်မေးပေးပါ\nသူတို့ရဲ့ရုံးချိန် 9:00AM to 3:00AM အတွင်းမှာ ပြန်လည်ဖြေကြားပေးသွားပါလိမ့်မယ်"
        }
        "one_aed" => &format!(
            "မြန်မာပြည်ငွေလွှဲ 1 AED ကို
{} MMK (Bank account transfer/Kpay/Wave/True Money)
{} MMK (Cash out at Ygn Office/Cash Home Delivery/ ဘဏ်ထုတ်)
ရပါမယ် − ငွေစျေးပြောင်းနိုင်ပါတယ် - *ငွေပြန်လွှဲပေးတဲ့အခါမှာ 1AED နဲ့တွက်ပြီးမလွှဲပေးပါဘူးနော်*
မှတ်ပုံတင်ဖြင့်လွှဲသည်ဖြစ်စေ နယ်အကောင့်များသို့ထည့်သည်ဖြစ်စေ ကျသင့်မည့် ဘဏ် charges များကို customer ဘက်မှသာကျခံပေးရပါမည်
အိမ်ပို့ငွေများအားလုံး 4-5 ရက်အထိကြာနိုင်ပါတယ်
ငွေမလွှဲခင်တိုင်းအကောင့်ပြန် confirmပြီးမှလွှဲပေးပါနော်",
            (100000.0 / rates[0]).round() as i32,
            (100000.0 / rates[1]).round() as i32
        ),
        "aed_mmk" => &format!(
            "မြန်မာပြည်ငွေလွှဲ 100,000 MMK ကို
{} AED (Bank account transfer/Kpay/Wave/True Money)
{} AED (Cash out at Ygn Office/Cash Home Delivery/ ဘဏ်ထုတ်)
ပေးရပါမယ် − ငွေစျေးပြောင်းနိုင်ပါတယ်
မှတ်ပုံတင်ဖြင့်လွှဲသည်ဖြစ်စေ နယ်အကောင့်များသို့ထည့်သည်ဖြစ်စေ ကျသင့်မည့် ဘဏ် charges များကို customer ဘက်မှသာကျခံပေးရပါမည်
အိမ်ပို့ငွေများအားလုံး 4-5 ရက်အထိကြာနိုင်ပါတယ်
ငွေမလွှဲခင်တိုင်းအကောင့်ပြန် confirmပြီးမှလွှဲပေးပါနော်",
            rates[0], rates[1]
        ),
        "mmk_aed" => &format!(
            "မြန်မာပြည်ကနေငွေလွဲမယ်ဆိုရင် တစ်သိန်းကို {} AED ရပါမယ် စျေးပြောင်းနိုင်ပါတယ်
AED ချက်ချင်းပြန်လိုတယ်ဆိုရုံးကို လာထုတ်ပေးမှရပါမယ် အကောင့်ထဲကို လွှဲပေးရမယ်ဆို up to 48 hr အထိကြာနိုင်ပါတယ်",
            rates[2]
        ),
        _ => "No available reply",
    };
    reply.to_string()
}
//==================================================================================================
//types
type RepliedCache = Arc<Mutex<HashMap<String, HashSet<String>>>>;
//constants
const BLOCKED: [&str; 15] = [
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
    //"135093918113861",      //KZL
    "17867584094214@lid",  //Phyo MaungMaung
    "200992322777123@lid", //TTW
    "102723068424257@lid", //KTDL Oman
                           //"199007410372652:22@lid" //ONLINE WA
                           //"80350717968563@lid", //Me
];
const ACCOUNT: [&str; 3] = ["acc", "account", "အကောင့်"];
const GREETING: [&str; 7] = ["hi", "hello", "မင်္ဂလာ", "မဂလာ", "မဂ်လာ", "xi", "ဟိုင်း"];
const TICKETING: [&str; 5] = ["ticketing", "ticket", "visa", "လက်မှတ်", "ဗီဇာ"];
const PASSPORT: [&str; 3] = ["passport", "ပတ်စပို့", "သက်တမ်းတိုး"];
const FOOD: [&str; 3] = ["food", "စားစရာ", "မုန့်"];
const ONE_AED: [&str; 4] = ["1aed", "1 ကျပ်", "1 aed", "1ကျပ်"];
const AED_MMK_RATE: [&str; 22] = [
    "aed to mmk",
    "aedtommk",
    "aed tommk",
    "aedto mmk",
    "aed to mk",
    "aedtomk",
    "aed tomk",
    "aedto mk",
    "aed mmk",
    "aed mk",
    "aedmmk",
    "aedmk",
    "aed to myanmar",
    "aedto myanmar",
    "aed tomyanmar",
    "aedtomyanmar",
    "aed to kpay",
    "aedto kpay",
    "aed tokpay",
    "aedtokpay",
    "မြန်မာပြည်ကို",
    "မြန်မာကိုလွှဲ",
];
const MMK_AED_RATE: [&str; 10] = [
    "mmk to aed",
    "mmk aed",
    "mmk→aed",
    "mk to aed",
    "mk aed",
    "ဒီဘက်မှာaed",
    "မြန်မာကနေ",
    "မြန်မာပြည်ကနေ",
    "kpay to aed",
    "မြန်မာငွေပေးပီး aed ယူ",
];
const CAPTION: &str = "ပုံထဲမှာ ရေးထားတာလေးသေချာသတိထားပေးပါ ကျေးဇူးတင်ပါတယ်\nလွှဲထားတဲ့စလစ်လေးကို နေ့တိုင်းမနက် 6:00AM မှာငွေစစ်ပြီးတော့ ငွေဝင်ထားတဲ့နေ့မှာမြန်မာပြည်ကနေပြန်လွှဲပေးပါတယ်";

static RATES: OnceLock<Vec<f64>> = OnceLock::new();
static START_TIME: OnceLock<i64> = OnceLock::new();
static AED_ACC: OnceLock<Vec<u8>> = OnceLock::new();
