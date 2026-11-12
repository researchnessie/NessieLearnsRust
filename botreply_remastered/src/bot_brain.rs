use crate::storage_manager;
use colored::*;
use regex::Regex;
use console_utils::input::input;
use std::time::{SystemTime, UNIX_EPOCH};
//constants
//these constants are for checking and matching specific keywords from the customer and to trigger the related replies
//these may be replaced with an onnx later on (big claim :-) )
pub const BLOCKED: [&str; 16] = {[
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
const AED_REG:&str = r"(?i)(?-u:\b)(aed|dubai|uae|ဒူဘိုင်း|rate|ငွေ|exchange|ဒီနေ့|today|1)\s*(to)?\s*(-)?\s*(ကနေ)?\s*(က)?\s*(-)?\s*(mmk|mk|kyat|myanmar|kpay|k pay|wave|မြန်မာ|rate|စျေး|ဘယ်လောက်|aed|ကျပ်)";
const MMK_REG:&str = r"(?i)(?-u:\b)(mmk|mk|kyat|myanmar|kpay|k pay|wave|မြန်မာ)\s*(to)?\s*(ကနေ)?\s*(က)?\s*(-)?\s*(aed|dubai|uae|ဒူဘိုင်း)";
pub const CAPTION: &str =   "ပုံထဲမှာ ရေးထားတာလေးသေချာသတိထားပေးပါ ကျေးဇူးတင်ပါတယ်\nလွှဲထားတဲ့စလစ်လေးကို နေ့တိုင်းမနက် 6:00AM မှာငွေစစ်ပြီးတော့ ငွေဝင်ထားတဲ့နေ့မှာမြန်မာပြည်ကနေပြန်လွှဲပေးပါတယ်";
//==================================================================================================
//enums
//These are the keys that the bot will be trying to match, and they are going to point to the correct reply
#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub enum MessageKey{
    Greeting,Ticketing,Passport,Food,AedMmk,MmkAed,Acc
}
//==================================================================================================
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
            Ok(v) if v.is_finite() && v.is_sign_positive() => return v,
            _ => {println!("{}","Invalid input, try again!".red())}
        }
    }
}
fn ask_mmk_aed_50under()->bool{
    loop{
        let x:String = input("50L limit on MMK_AED? : ");
        match x.trim().parse::<bool>(){
            Ok(v) => return v,
            _ => {println!("{}","Invalid input, try again!".red())}
        }
    }
}
//==================================================================================================
//This function is created to safely rotate through the keywords and to discard Burmese text gluing to English characters
fn is_glue(c: char) -> bool {c.is_ascii_alphanumeric()}
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
//==================================================================================================
//the Brain of the bot
//This is where all the reply are matched and sent, the actual key matching part, well i need to think if i want to move that here as well
pub struct Brain{
    rates:[f64;3], mmk_transfer_limit:bool,
    pub start_time:i64, pub aed_acc:Vec<u8>,
    aed_reg:Regex, mmk_reg: Regex
}
impl Brain{
    pub fn init() -> Self{
        println!("{}", "Type your rates...".green());
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        Self{
            rates:[ask_aed_acc(),ask_aed_cash(),ask_mmk_aed()],
            mmk_transfer_limit:ask_mmk_aed_50under(),
            start_time:now,
            aed_acc: storage_manager::load_picture("aedpic.jpeg"),
            aed_reg:Regex::new(AED_REG).unwrap(),
            mmk_reg:Regex::new(MMK_REG).unwrap()
        }
    }
    pub fn get_matching_keys(&self,message:&String)->Vec<MessageKey>{
        let mut keys_to_reply:Vec<MessageKey> = Vec::new();
        if GREETING.iter().any(|&word| has_keyword(&message,word)) {keys_to_reply.push(MessageKey::Greeting)}
        if TICKETING.iter().any(|&word| has_keyword(&message, word)) {keys_to_reply.push(MessageKey::Ticketing)}
        if PASSPORT.iter().any(|&word| has_keyword(&message,word)) {keys_to_reply.push(MessageKey::Passport)}
        if FOOD.iter().any(|&word| has_keyword(&message,word)) {keys_to_reply.push(MessageKey::Food)}
        if self.aed_reg.is_match(&message){keys_to_reply.push(MessageKey::AedMmk)}
        if self.mmk_reg.is_match(&message){keys_to_reply.push(MessageKey::MmkAed)}
        if ACCOUNT.iter().any(|&word| has_keyword(&message,word)) {keys_to_reply.push(MessageKey::Acc)}
        keys_to_reply
    }
    pub fn get_reply(&self,key:MessageKey)->Option<String>{
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
            MessageKey::AedMmk => {
                Some(format!("မြန်မာပြည်ငွေလွှဲ 100,000 MMK/ 1 AED ကို\n{} AED/ {} MMK (Bank account transfer/Kpay/Wave/True Money)\n{} AED/ {}MMK (Cash out at Ygn Office/Cash Home Delivery/ ဘဏ်ထုတ်)\nပေးရပါမယ် − ငွေစျေးပြောင်းနိုင်ပါတယ် - *ငွေပြန်လွှဲပေးတဲ့အခါမှာ 1AED နဲ့တွက်ပြီးမလွှဲပေးပါဘူးနော်*\nမှတ်ပုံတင်ဖြင့်လွှဲသည်ဖြစ်စေ နယ်အကောင့်များသို့ထည့်သည်ဖြစ်စေ ကျသင့်မည့် ဘဏ် charges များကို customer ဘက်မှသာကျခံပေးရပါမည်\nအိမ်ပို့ငွေများအားလုံး 4-5 ရက်အထိကြာနိုင်ပါတယ်\nငွေမလွှဲခင်တိုင်းအကောင့်ပြန် confirmပြီးမှလွှဲပေးပါနော်",
                             self.rates[0],(100000.0/self.rates[0]).round() as i32, self.rates[1], (100000.0/self.rates[1]).round() as i32))
            }
            MessageKey::MmkAed => {
                if self.rates[2] == 0.0{
                    Some("လောလောဆယ် mmk to aed ငွေလွှဲပေးရန် stock မရှိသေးပါဘူး ငွေပြန်လွှဲလို့ရတဲ့အခါပြောပေးပါမယ်နော်".to_string())
                }
                else{
                    if self.mmk_transfer_limit {
                        Some(format!("မြန်မာပြည်ကနေငွေလွဲမယ်ဆိုရင် တစ်သိန်းကို {} AED ရပါမယ် စျေးပြောင်းနိုင်ပါတယ်\nAED ချက်ချင်းပြန်လိုတယ်ဆိုရုံးကို လာထုတ်ပေးမှရပါမယ် အကောင့်ထဲကို လွှဲပေးရမယ်ဆို up to 48 hr အထိကြာနိုင်ပါတယ်\n\n**လောလောဆယ်stockအများကြီးမရှိတဲ့အတွက်မြန်မာငွေသိန်း50အောက်ပဲငွေလွှဲလက်ခံပါတယ်**",self.rates[2]))
                    }
                    else{
                        Some(format!("မြန်မာပြည်ကနေငွေလွဲမယ်ဆိုရင် တစ်သိန်းကို {} AED ရပါမယ် စျေးပြောင်းနိုင်ပါတယ်\nAED ချက်ချင်းပြန်လိုတယ်ဆိုရုံးကို လာထုတ်ပေးမှရပါမယ် အကောင့်ထဲကို လွှဲပေးရမယ်ဆို up to 48 hr အထိကြာနိုင်ပါတယ်",self.rates[2]))
                    }
                }
            }
            MessageKey::Acc => None
        }
    }
}