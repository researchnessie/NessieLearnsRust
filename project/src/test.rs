use colored::Colorize;
use console_utils::input::input;
use regex::Regex;

pub fn test(){
    let catcher = Regex::new(r"(?i)\b(aed|dubai|uae|ဒူဘိုင်း|)\s*(to)?\s*(ကနေ)?\s*(က)?\s*(mmk|mk|myanmar|kpay|wave|မြန်မာ|)").unwrap();
    loop {let x:String = input("anything!");
    if catcher.is_match(&x) {println!("{}", "it is a match".green());}
    else{println!("{}", "it is not a match".red());}}
}