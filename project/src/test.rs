use console_utils::input::input;
pub fn test(){
    let checker:String = input("enter: ");
    for c in TICKETING.iter(){
        if c.to_string() == checker.trim(){println!("Yes in the list!")}
    }
    test()
}
const TICKETING: [&str;5] = ["ticketing", "ticket", "visa", "လက်မှတ်", "ဗီဇာ"];
const PASSPORT: [&str;3] = ["passport", "ပတ်စပို့", "သက်တမ်းတိုး"];
const FOOD: [&str;3] = ["food", "စားစရာ", "မုန့်"];
const ONE_AED:[&str;4] = ["1aed","1 ကျပ်", "1 aed", "1ကျပ်"];