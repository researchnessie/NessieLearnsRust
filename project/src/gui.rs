//Import modules
use walkdir::WalkDir;
use arboard::{ImageData, Clipboard};
use rsautogui::{keyboard, keyboard::Vk};
use std::{thread, time::Duration, rc::Rc, cell::RefCell, path::PathBuf, process::exit};
use fltk::{app, button::Button, frame::Frame, prelude::*, window::Window, input::Input, enums::Color};
//==================================================================================================
//global functions
fn img_data(location:&str, filename:&str)->ImageData<'static>{
    let Some(acc) = find_file(location,filename) else {exit(0)};
    let picture = image::open(acc).unwrap().to_rgba8();
    ImageData {
        width:picture.width() as usize,
        height:picture.height() as usize,
        bytes:picture.into_raw().into()
    }
}
fn find_file(location:&str, filename:&str) -> Option<PathBuf>{
    for entry in WalkDir::new(location) {
        let Ok(entry) = entry else {continue};
        if entry.file_name() == filename && entry.file_type().is_file() {
            return Some(entry.path().to_path_buf())
        }
    }
    None
}
fn wait(seconds:f64){thread::sleep(Duration::from_secs(seconds as u64));}
fn next_line(){
    keyboard::key_down(Vk::Shift);
    keyboard::key_tap(Vk::Enter);
    keyboard::key_up(Vk::Shift);
}
fn do_paste_action(){
    keyboard::key_down(Vk::Control);
    keyboard::key_tap(Vk::V);
    keyboard::key_up(Vk::Control);
    wait(1.0);
    keyboard::key_tap(Vk::Enter);
    wait(1.0);
}
pub fn gui(){
    //global shared variables
    let aed_acc = img_data("C:\\Users", "aedpic.jpeg");
    let oman_acc = img_data("C:\\Users", "omanpic.jpeg");
    let kpay_acc = img_data("C:\\Users", "kpaypic.jpeg");
    let location_1 = img_data("C:\\Users", "location_1.jpeg");
    let location_2 = img_data("C:\\Users", "location_2.jpeg");
    let final_rates: Rc<RefCell<Vec<f64>>> = Rc::new(RefCell::new(Vec::new()));
    let rates_to_update = Rc::clone(&final_rates);
    let rates_for_aed_mmk = Rc::clone(&final_rates);
    let rates_for_mmk_aed = Rc::clone(&final_rates);
    let rates_for_one_aed = Rc::clone(&final_rates);
    //==============================================================================================
    //make the window
    let app = app::App::default();
    let mut wind = Window::new(1000, 200, 800, 800,"This is windows title!");
    //==============================================================================================
    //add inputs
    let mut aed1 = Input::new(100,100,50,20, "AED_Acc");
    let mut aed2 = Input::new(300,100,50,20,"AED_Cash");
    let mut mmk = Input::new(500,100,50,20,"MMK");
    aed1.hide();aed2.hide();mmk.hide();
    let mut aed1_show = aed1.clone();
    let mut aed2_show = aed2.clone();
    let mut mmk_show = mmk.clone();
    //==============================================================================================
    //non-changing widgets
    let mut title_frame = Frame::new(200,0,100,20,"Fast Reply Standard Edition");
    title_frame.set_label_size(25);
    title_frame.set_label_color(Color::White);
    title_frame.set_color(Color::DarkYellow);
    title_frame.show();
    let mut warn_frame = Frame::new(200,20,100,20,"All MMK are in lakhs!");
    warn_frame.set_label_size(10);
    warn_frame.set_label_color(Color::Red);
    let mut cra_frame = Frame::new(20,40,100,20,"Current rates:");
    cra_frame.set_label_size(12);
    Frame::new(100,60,40,20,"AED_Acc   "); //let aed_acc_frame =
    Frame::new(300,60,40,20,"AED_Cash   ");//let aed_cash_frame =
    Frame::new(500,60,40,20,"MMK   ");//let mmk_frame =
    //==============================================================================================
    //update-able frames
    let mut frame1 = Frame::new(150,60,30,20,"");
    let mut frame2 = Frame::new(350,60,30,20,"");
    let mut frame3 = Frame::new(550,60,30,20,"");
    //==============================================================================================
    //add buttons
    let mut exit_ = Button::new(400, 780, 50, 20, "Exit");
    let mut type_rates = Button::new(100,20,50,20,"Type Rates");
    let mut confirm_rates = Button::new(0, 0, 50, 20, "Confirm");

    //Buttons
    let mut aed_mmk = Button::new(50, 200, 100, 25, "AED_MMK");
    let mut mmk_aed = Button::new(50, 250, 100, 25, "MMK_AED");
    let mut one_aed = Button::new(50, 300, 100, 25, "1AED");

    let mut current_aed_acc = Button::new(200, 200, 100, 25, "AED_Acc");
    let mut current_oman_acc = Button::new(200, 250, 100, 25, "Oman_Acc");
    let mut dubai_kpay_acc = Button::new(200, 300, 100, 25, "Dubai_Kpay");
    let mut ygn_kpay_acc = Button::new(200, 350, 100, 25, "YGN_Kpay");
    let mut ygn_wave_acc = Button::new(200, 400, 100, 25, "YGN_Wave");
    let mut ygn_bank_acc = Button::new(200, 450, 100, 25, "YGN_Banks");

    let mut transfer_process = Button::new(350, 200, 100, 25, "Transfer_process");
    let mut delay_48hr = Button::new(350, 250, 100, 25, "48Hr_delay");
    let mut ask_acc = Button::new(350, 300, 100, 25, "Ask_account");
    let mut large_amt = Button::new(350, 350, 100, 25, "Large_amount");
    let mut kpay_limit = Button::new(350, 400, 100, 25, "Kpay_limit");
    let mut dubai_location = Button::new(350, 450, 100, 25, "Dubai_Location");

    let mut ticket = Button::new(500, 200, 100, 25, "Ticket");
    let mut passport = Button::new(500, 250, 100, 25, "Passport");
    let mut office = Button::new(500, 300, 100, 25, "Office");
    let mut food = Button::new(500, 350, 100, 25, "Food");
    //==============================================================================================
    //exit function
    exit_.set_callback(move |_| {exit(0)});
    //updates rates to labels
    type_rates.set_callback(move |_|{
        aed1_show.show();
        aed2_show.show();
        mmk_show.show();
    });
    confirm_rates.set_callback(move |_| {
        let got_rates = get_rates(&aed1.value(), &aed2.value(), &mmk.value());
        frame1.set_label(&format!("{}", got_rates[0]));
        frame2.set_label(&format!("{}", got_rates[1]));
        frame3.set_label(&format!("{}", got_rates[2]));
        *rates_to_update.borrow_mut() = got_rates;
        aed1.hide();aed2.hide();mmk.hide();
    });

    //calculation function buttons
    aed_mmk.set_callback(move |_| {
        let i = rates_for_aed_mmk.borrow_mut();
        wait(3.0);
        keyboard::typewrite("မြန်မာပြည်ငွေလွှဲ 100,000 MMK ကို");next_line();
        keyboard::typewrite(&format!("{} AED (Bank account transfer/Kpay/Wave/True Money)", i[0]));next_line();
        keyboard::typewrite(&format!("{} AED (Cash out at Ygn Office/Cash Home Delivery/ ဘဏ်ထုတ်", i[1]));next_line();
        keyboard::typewrite("ပေးရပါမယ် − ငွေစျေးပြောင်းနိုင်ပါတယ်");next_line();
        keyboard::typewrite("မှတ်ပုံတင်ဖြင့်လွှဲသည်ဖြစ်စေ နယ်အကောင့်များသို့ထည့်သည်ဖြစ်စေ ကျသင့်မည့် ဘဏ် charges များကို customer ဘက်မှသာကျခံပေးရပါမည်");next_line();
        keyboard::typewrite("အိမ်ပို့ငွေများအားလုံး 4-5 ရက်အထိကြာနိုင်ပါတယ်");next_line();
        keyboard::typewrite("ငွေမလွှဲခင်တိုင်းအကောင့်ပြန် confirmပြီးမှလွှဲပေးပါနော်");
        keyboard::key_tap(Vk::Enter);
    });
    mmk_aed.set_callback(move |_| {
        let i = rates_for_mmk_aed.borrow_mut()[2];
        wait(3.0);
        keyboard::typewrite(&format!("မြန်မာပြည်ကနေငွေလွဲမယ်ဆိုရင် တစ်သိန်းကို {} AED ရပါမယ် စျေးပြောင်းနိုင်ပါတယ်",i));next_line();
        keyboard::typewrite(r"AED ချက်ချင်းပြန်လိုတယ်ဆိုရုံးကို လာထုတ်ပေးမှရပါမယ် အကောင့်ထဲကို လွှဲပေးရမယ်ဆို up to 48 hr အထိကြာနိုင်ပါတယ်");
        keyboard::key_tap(Vk::Enter);
    });
    one_aed.set_callback(move |_| {
        let i = rates_for_one_aed.borrow_mut();
        wait(3.0);
        keyboard::typewrite("မြန်မာပြည်ငွေလွှဲ 1 AED ကို");next_line();
        keyboard::typewrite(&format!("{} MMK (Bank account transfer/Kpay/Wave/True Money)", (100000.0/i[0]).round() as i32));next_line();
        keyboard::typewrite(&format!("{} MMK (Cash out at Ygn Office/Cash Home Delivery/ ဘဏ်ထုတ်", (100000.0/i[1]).round() as i32));next_line();
        keyboard::typewrite("ငွေစျေးပြောင်းနိုင်ပါတယ်- *ငွေပြန်လွှဲပေးတဲ့အခါမှာ 1AED နဲ့တွက်ပြီးမလွှဲပေးပါဘူးနော်*");next_line();
        keyboard::typewrite("မှတ်ပုံတင်ဖြင့်လွှဲသည်ဖြစ်စေ နယ်အကောင့်များသို့ထည့်သည်ဖြစ်စေ ကျသင့်မည့် ဘဏ် charges များကို customer ဘက်မှသာကျခံပေးရပါမည်");next_line();
        keyboard::typewrite("ငွေမလွှဲခင်တိုင်းအကောင့်ပြန် confirmပြီးမှလွှဲပေးပါနော်");
        keyboard::key_tap(Vk::Enter);
    });
    //static function buttons
    current_aed_acc.set_callback(move |_| {
        Clipboard::new().unwrap().set_image(aed_acc.clone()).unwrap();
        wait(3.0);
        do_paste_action();
        keyboard::typewrite("ပုံထဲမှာ ရေးထားတာလေးသေချာသတိထားပေးပါ ကျေးဇူးတင်ပါတယ်");
        next_line();
        keyboard::typewrite("လွှဲထားတဲ့စလစ်လေးကို နေ့တိုင်းမနက် 6:00AM မှာငွေစစ်ပြီးတော့ ငွေဝင်ထားတဲ့နေ့မှာမြန်မာပြည်ကနေပြန်လွှဲပေးပါတယ်");
        keyboard::key_tap(Vk::Enter);
    });
    current_oman_acc.set_callback(move |_| {
        Clipboard::new().unwrap().set_image(oman_acc.clone()).unwrap();
        wait(3.0);
        do_paste_action();
        keyboard::typewrite("ဒီအကောင့်ကိုလွှဲလို့ရပါတယ် ငွေလွှဲအကြောင်းအရာမှာ Family Support လို့လုံးဝမရေးပါနဲ့ ကျေးဇူးတင်ပါတယ်");
        keyboard::key_tap(Vk::Enter);
    });
    dubai_kpay_acc.set_callback(move |_| {
        Clipboard::new().unwrap().set_image(kpay_acc.clone()).unwrap();
        wait(3.0);
        do_paste_action();
        keyboard::typewrite("ဒီKpayအကောင့်တွေထဲက အဆင်ပြေတာကိုလွှဲလို့ရပါတယ်");
        keyboard::key_tap(Vk::Enter);
    });
    ygn_kpay_acc.set_callback(move |_| {
        wait(3.0);
        keyboard::typewrite("Mya Mya Aye - 09421018434");next_line();
        keyboard::typewrite("Khin Maung Than - 095505394");next_line();
        keyboard::typewrite("Thazin Moe - 09785505394");next_line();
        keyboard::typewrite("Kpay");keyboard::key_tap(Vk::Enter);
        keyboard::typewrite("အခုပို့ပေးထားတဲ့အကောင့်တွေက ရန်ကုန်ရုံးက အကောင့်တွေပါ အဲ့တာကြောင့် ရုံးကိုမေးပြီးတော့ ငွေဝင်မဝင် တစ်ဆင့်ငွေစစ်ရပါတယ် ငွေဝင်ထားကြောင်း confirm တဲ့အခါ ကျတော်တို့ဘက်ကပြန်ပြောပေးပါမယ် အဲ့အခါကျမှလာထုတ်ပေးပါ မဟုတ်ရင် ငွေလာထုတ်တဲ့အခါ ရုံးမှာ အကြာကြီးစောင့်နေရမှာမလို့ပါ");
        keyboard::key_tap(Vk::Enter);
    });
    ygn_wave_acc.set_callback(move |_| {
        wait(3.0);
        keyboard::typewrite("Shwe Maw Htoo");next_line();
        keyboard::typewrite("09958580157 Wave account");
        keyboard::key_tap(Vk::Enter);
        keyboard::typewrite("အခုပို့ပေးထားတဲ့အကောင့်က ရန်ကုန်ရုံးက အကောင့်ပါ အဲ့တာကြောင့် ရုံးကိုမေးပြီးတော့ ငွေဝင်မဝင် တစ်ဆင့်ငွေစစ်ရပါတယ် ငွေဝင်ထားကြောင်း confirm တဲ့အခါ ကျတော်တို့ဘက်ကပြန်ပြောပေးပါမယ် အဲ့အခါကျမှလာထုတ်ပေးပါ မဟုတ်ရင် ငွေလာထုတ်တဲ့အခါ ရုံးမှာ အကြာကြီးစောင့်နေရမှာမလို့ပါ");
        keyboard::key_tap(Vk::Enter)
    });
    ygn_bank_acc.set_callback(move |_| {
        wait(3.0);
        keyboard::typewrite("Name - Mya Mya Aye");next_line();
        keyboard::typewrite("==============================");next_line();
        keyboard::typewrite("CB - 0048100900006009");next_line();
        keyboard::typewrite("Yoma - 005545498000784");next_line();
        keyboard::typewrite("AYA - 40028269966");next_line();
        keyboard::typewrite("KBZ Special Acc- 22851100203893301");
        keyboard::key_tap(Vk::Enter);
        keyboard::typewrite("အခုပို့ပေးထားတဲ့အကောင့်တွေက ရန်ကုန်ရုံးက အကောင့်တွေပါ အဲ့တာကြောင့် ရုံးကိုမေးပြီးတော့ ငွေဝင်မဝင် တစ်ဆင့်ငွေစစ်ရပါတယ် ငွေဝင်ထားကြောင်း confirm တဲ့အခါ ကျတော်တို့ဘက်ကပြန်ပြောပေးပါမယ် အဲ့အခါကျမှလာထုတ်ပေးပါ မဟုတ်ရင် ငွေလာထုတ်တဲ့အခါ ရုံးမှာ အကြာကြီးစောင့်နေရမှာမလို့ပါ");
        keyboard::key_tap(Vk::Enter);
    });

    transfer_process.set_callback(move |_| {
        wait(3.0);
        keyboard::typewrite("လွှဲထားတဲ့စလစ်လေးကို နေ့တိုင်းမနက် 6:00AM မှာငွေစစ်ပြီးတော့ ငွေဝင်ထားတဲ့နေ့မှာမြန်မာပြည်ကနေပြန်လွှဲပေးပါတယ်");
        keyboard::key_tap(Vk::Enter);
    });
    delay_48hr.set_callback(move |_| {
        wait(3.0);
        keyboard::typewrite("AED ချက်ချင်းပြန်လိုတယ်ဆိုရုံးကို လာထုတ်ပေးမှရပါမယ် အကောင့်ထဲကို လွှဲပေးရမယ်ဆို up to 48 hr အထိကြာနိုင်ပါတယ်");
        keyboard::key_tap(Vk::Enter);
    });
    ask_acc.set_callback(move |_| {
        wait(3.0);
        keyboard::typewrite("ငွေမလွှဲခင် အကောင့်ပြန်မေးပေးပါ ပိတ်နေပျက်နေတဲ့အကောင့်တွေကိုမှားလွှဲမိလို့ငွေဆုံးရုံးပါက TNT မှာတာဝန်ယူဖြေရှင်းပေးမည်မဟုတ်ပါ");
        keyboard::key_tap(Vk::Enter);
    });
    large_amt.set_callback(move |_| {
        wait(3.0);
        keyboard::typewrite("လွှဲမယ့်အမောင့်လေးရယ် AED ပေးမယ့်ပုံစံရယ် MMK ပြန်ယူမယ့်ပုံစံရယ်ပြောပေးပါနော် boss ကိုစျေးမေးပေးပါမယ်");
        keyboard::key_tap(Vk::Enter);
    });
    kpay_limit.set_callback(move |_| {
        wait(3.0);
        keyboard::typewrite("kpay က တစ်နေ့ကို 5,000,000 MMK ပဲလွှဲပေးလို့ရပါတယ် ကျန်တာ နောက်နေ့မှထပ်ထည့်ပေးလို့ရပါမယ် ဒီနေ့လိုချင်တယ်ဆို နောက်ထပ်အကောင့်တစ်ခုထပ်ပေးမှရပါမယ်");
        keyboard::key_tap(Vk::Enter);
    });
    dubai_location.set_callback(move |_| {
        wait(3.0);
        Clipboard::new().unwrap().set_image(location_1.clone()).unwrap();
        do_paste_action();
        Clipboard::new().unwrap().set_image(location_2.clone()).unwrap();
        do_paste_action();
        keyboard::typewrite("https://maps.app.goo.gl/Ji8WJeeJMVAGMZCs8");
        keyboard::key_tap(Vk::Enter);
    });

    ticket.set_callback(move |_| {
        wait(3.0);
        keyboard::typewrite("Visa and Ticket နဲ့ပတ်သက်ပြီးသိလိုတာတွေကို Visa and Ticketing WhatsApp +971501238266 ကိုဆက်သွယ်မေးပေးပါ");
        next_line();
        keyboard::typewrite("သူတို့ရဲ့ရုံးချိန် 9:00AM to 12:00AM(Midnight) အတွင်းမှာ ပြန်လည်ဖြေကြားပေးသွားပါလိမ့်မယ်");
        keyboard::key_tap(Vk::Enter);
    });
    passport.set_callback(move |_| {
        wait(3.0);
        keyboard::typewrite("Passport နဲ့ပတ်သက်ပြီးသိလိုတာတွေကို Office Reception WhatsApp +971501775318 ကိုဆက်သွယ်မေးပေးပါ");
        next_line();
        keyboard::typewrite("သူတို့ရဲ့ရုံးချိန် 9:00AM to 3:00AM အတွင်းမှာ ပြန်လည်ဖြေကြားပေးသွားပါလိမ့်မယ်");
        keyboard::key_tap(Vk::Enter);
    });
    office.set_callback(move |_| {
        wait(3.0);
        keyboard::typewrite("TNT Office မှာငွေလွှဲထားတယ်ဆိုရင် Office Reception WhatsApp +971501775318 ကိုဆက်သွယ်မေးပေးပါ");
        next_line();
        keyboard::typewrite("သူတို့ရဲ့ရုံးချိန် 9:00AM to 3:00AM အတွင်းမှာ ပြန်လည်ဖြေကြားပေးသွားပါလိမ့်မယ်");
        keyboard::key_tap(Vk::Enter);
    });
    food.set_callback(move |_| {
        wait(3.0);
        keyboard::typewrite("မြန်မာအစားစာ နဲ့ပတ်သက်ပြီးသိလိုတာတွေကို TNT Food WhatsApp +971503490683 ကိုဆက်သွယ်မေးပေးပါ");
        next_line();
        keyboard::typewrite("သူတို့ရဲ့ရုံးချိန် 9:00AM to 3:00AM အတွင်းမှာ ပြန်လည်ဖြေကြားပေးသွားပါလိမ့်မယ်");
        keyboard::key_tap(Vk::Enter);
    });
    //==============================================================================================
    wind.end();
    wind.show();
    app.run().unwrap();
    //==============================================================================================
    //backend
    fn get_rates(a:&str,b:&str,c:&str)-> Vec<f64>{
        let rates_to_check =vec![a.to_string(),b.to_string(),c.to_string()];
        let mut ready_rates = vec![0.0];
        ready_rates.pop();
        for r in rates_to_check.iter(){
            match r.parse::<f64>(){
                Ok(_) => ready_rates.push(r.parse().unwrap()),
                Err(_)=> ready_rates.push(0.0),
            }
        }
        if ready_rates.iter().any(|r| *r < 70.0){
            return vec![0.0,0.0,0.0];
        }
        ready_rates
    }
    //==============================================================================================
}