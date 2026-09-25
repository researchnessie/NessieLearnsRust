//Import modules
use walkdir::WalkDir;
use arboard::{ImageData, Clipboard};
use num_format::{Locale, ToFormattedString};
use rsautogui::{keyboard, keyboard::Vk};
use std::{thread, time::Duration, rc::Rc, cell::RefCell, process::exit};
use fltk::{app, button::Button, frame::Frame, prelude::*, window::Window, input::Input, enums::{FrameType,Align,Color}, group::{Flex, FlexType}, image::PngImage};
//==================================================================================================
//global functions
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
fn missing_picture_popup(filename: &str) -> ! {
    let msg = format!(
        "{} picture failed to acquire, make sure it's in the Downloads folder and the file isn't corrupted, then re-launch the app!",
        filename
    );
    let mut warn = Window::new(300, 150, 420, 170, "Missing Picture");
    style_window(&mut warn);
    let mut warn_root = Flex::new(0, 0, 420, 170, "");
    warn_root.set_type(FlexType::Column);
    warn_root.set_margin(15);
    warn_root.set_pad(10);
    let mut warn_msg = Frame::new(0, 0, 0, 0, msg.as_str());
    warn_msg.set_label_color(Color::from_rgb(255, 99, 99));
    warn_msg.set_align(Align::Wrap | Align::Center);
    let mut ok = Button::new(0, 0, 0, 0, "Ok");
    style_button(&mut ok);
    warn_root.fixed(&ok, 30);
    warn_root.end();
    warn.resizable(&warn_root);
    warn.make_modal(true);
    warn.end();
    warn.show();
    ok.set_callback(move |_| { exit(0); });
    loop { app::wait(); }
}//blocks until the user acknowledges, then exits the whole program
fn load_or_warn(filename: &str) -> ImageData<'static> {
    match process_images(filename) {
        Some(data) => data,
        None => missing_picture_popup(filename),
    }
}//convenience wrapper used at startup
fn wait(seconds:f64){thread::sleep(Duration::from_secs(seconds as u64));}//simplifying wait for multiple uses
fn next_line(){
    keyboard::key_down(Vk::Shift);
    keyboard::key_tap(Vk::Enter);
    keyboard::key_up(Vk::Shift);
}//performs Shift Enter for WhatsApp and Messenger
fn do_paste_action(){
    keyboard::key_down(Vk::Control);
    keyboard::key_tap(Vk::V);
    keyboard::key_up(Vk::Control);
    wait(1.0);
    keyboard::key_tap(Vk::Enter);
    wait(1.0);
}//simulating paste picture and send picture action
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
}//solves user input error on typing rates
//==================================================================================================
//Visual changing functions
fn style_window(w: &mut Window) {
    w.set_color(Color::from_rgb(24, 24, 27));
}
fn style_button(b: &mut Button) {
    b.set_color(Color::from_rgb(45, 45, 50));
    b.set_label_color(Color::from_rgb(255, 192, 203));
    b.set_frame(FrameType::FlatBox);
    b.set_down_frame(FrameType::FlatBox);
}
fn style_input(i: &mut Input) {
    i.set_color(Color::from_rgb(32, 32, 36));
    i.set_text_color(Color::from_rgb(235, 235, 235));
    i.set_frame(FrameType::BorderBox);
}
//==================================================================================================
//Main Window
pub fn fast_reply(){
    // Make the APP
    let app = app::App::default();
    app::background(24, 24, 27);
    app::background2(32, 32, 36);
    app::foreground(230, 230, 230);

    //global shared variables
    let aed_acc = load_or_warn("aedpic.jpeg");
    let oman_acc = load_or_warn("omanpic.jpeg");
    let kpay_acc = load_or_warn("kpaypic.jpeg");
    let location_1 = load_or_warn("location_1.jpeg");
    let location_2 = load_or_warn("location_2.jpeg");
    let final_rates: Rc<RefCell<Vec<f64>>> = Rc::new(RefCell::new(Vec::new()));
    for _ in 1..4 {final_rates.borrow_mut().push(0.0);}
    let rates_to_update = Rc::clone(&final_rates);
    //==============================================================================================
    // Popup Window (rate entry)
    let mut rate_popup = Window::new(300, 300, 260, 190, "Enter Rates");
    style_window(&mut rate_popup);
    let mut popup_root = Flex::new(0, 0, 260, 190, "");
    popup_root.set_type(FlexType::Column);
    popup_root.set_margin(15);
    popup_root.set_pad(10);
    let mut aed1_row = Flex::new(0, 0, 0, 0, "");
    aed1_row.set_type(FlexType::Row);
    let aed1_label = Frame::new(0, 0, 0, 0, "AED_Acc");
    aed1_row.fixed(&aed1_label, 70);
    let mut aed1 = Input::new(0, 0, 0, 0, "");
    aed1_row.end();
    popup_root.fixed(&aed1_row, 30);

    let mut aed2_row = Flex::new(0, 0, 0, 0, "");
    aed2_row.set_type(FlexType::Row);
    let aed2_label = Frame::new(0, 0, 0, 0, "AED_Cash");
    aed2_row.fixed(&aed2_label, 70);
    let mut aed2 = Input::new(0, 0, 0, 0, "");
    aed2_row.end();
    popup_root.fixed(&aed2_row, 30);

    let mut mmk_row = Flex::new(0, 0, 0, 0, "");
    mmk_row.set_type(FlexType::Row);
    let mmk_label = Frame::new(0, 0, 0, 0, "MMK");
    mmk_row.fixed(&mmk_label, 70);
    let mut mmk = Input::new(0, 0, 0, 0, "");
    mmk_row.end();
    popup_root.fixed(&mmk_row, 30);
    style_input(&mut aed1); style_input(&mut aed2); style_input(&mut mmk);
    let mut confirm_rates = Button::new(0, 0, 0, 0, "Confirm");
    style_button(&mut confirm_rates);
    popup_root.fixed(&confirm_rates, 30);
    popup_root.end();
    rate_popup.resizable(&popup_root);
    rate_popup.make_modal(true);
    rate_popup.end();
    //==============================================================================================
    // Main Window
    let mut wind = Window::new(1000, 200, 900, 750, "FastReply");
    style_window(&mut wind);
    if let Ok(icon) = PngImage::load("assets/icon.png") {
        wind.set_icon(Some(icon));
    }

    let mut root = Flex::new(0, 0, 900, 750, "");
    root.set_type(FlexType::Column);
    root.set_margin(12);
    root.set_pad(8);

    //------------------------------------------------------------------------------------------
    // Header: title, warning, current-rates row
    let mut header = Flex::new(0, 0, 0, 0, "");
    header.set_type(FlexType::Column);
    header.set_pad(4);

    let mut title_row = Flex::new(0, 0, 0, 0, "");
    title_row.set_type(FlexType::Row);
    let mut title_frame = Frame::new(0, 0, 0, 0, "Fast Reply Standard Edition");
    title_frame.set_label_size(25);
    title_frame.set_color(Color::from_rgb(88, 101, 242));
    title_frame.set_label_color(Color::from_rgb(255, 192, 203));
    title_frame.show();
    let mut type_rates = Button::new(0, 0, 0, 0, "Type Rates");
    title_row.fixed(&type_rates, 110);
    title_row.end();
    header.fixed(&title_row, 40);

    let mut warn_frame = Frame::new(0, 0, 0, 0, "All MMK are in lakhs!");
    warn_frame.set_label_size(11);
    warn_frame.set_label_color(Color::Red);
    header.fixed(&warn_frame, 20);

    let mut rates_row = Flex::new(0, 0, 0, 0, "");
    rates_row.set_type(FlexType::Row);
    let mut cra_frame = Frame::new(0, 0, 0, 0, "Current rates:");
    cra_frame.set_label_color(Color::from_rgb(255, 192, 203));
    cra_frame.set_label_size(12);
    rates_row.fixed(&cra_frame, 100);
    let mut aed_acc_label = Frame::new(0, 0, 0, 0, "AED_Acc");
    rates_row.fixed(&aed_acc_label, 70);
    let mut frame1 = Frame::new(0, 0, 0, 0, "");
    rates_row.fixed(&frame1, 60);
    let mut aed_cash_label = Frame::new(0, 0, 0, 0, "AED_Cash");
    rates_row.fixed(&aed_cash_label, 70);
    let mut frame2 = Frame::new(0, 0, 0, 0, "");
    rates_row.fixed(&frame2, 60);
    let mut mmk_label = Frame::new(0, 0, 0, 0, "MMK");
    rates_row.fixed(&mmk_label, 50);
    let mut frame3 = Frame::new(0, 0, 0, 0, "");
    rates_row.fixed(&frame3, 60);
    rates_row.end();
    header.fixed(&rates_row, 30);

    for f in [&mut aed_acc_label, &mut aed_cash_label, &mut mmk_label] {
        f.set_label_color(Color::from_rgb(255, 192, 203));
    }
    for f in [&mut frame1, &mut frame2, &mut frame3] {
        f.set_label_color(Color::from_rgb(120, 220, 150));
    }

    header.end();
    root.fixed(&header, 100);

    //------------------------------------------------------------------------------------------
    // Button grid: 4 columns (fills whatever space is left over)
    let mut button_grid = Flex::new(0, 0, 0, 0, "");
    button_grid.set_type(FlexType::Row);
    button_grid.set_pad(10);

    let mut col_rates = Flex::new(0, 0, 0, 0, "");
    col_rates.set_type(FlexType::Column);
    col_rates.set_pad(6);
    let mut aed_mmk = Button::new(0, 0, 0, 0, "AED_MMK");
    let mut mmk_aed = Button::new(0, 0, 0, 0, "MMK_AED");
    let mut one_aed = Button::new(0, 0, 0, 0, "1AED");
    col_rates.end();

    let mut col_accounts = Flex::new(0, 0, 0, 0, "");
    col_accounts.set_type(FlexType::Column);
    col_accounts.set_pad(6);
    let mut current_aed_acc = Button::new(0, 0, 0, 0, "AED_Acc");
    let mut current_oman_acc = Button::new(0, 0, 0, 0, "Oman_Acc");
    let mut dubai_kpay_acc = Button::new(0, 0, 0, 0, "Dubai_Kpay");
    let mut ygn_kpay_acc = Button::new(0, 0, 0, 0, "YGN_Kpay");
    let mut ygn_wave_acc = Button::new(0, 0, 0, 0, "YGN_Wave");
    let mut ygn_bank_acc = Button::new(0, 0, 0, 0, "YGN_Banks");
    col_accounts.end();

    let mut col_info = Flex::new(0, 0, 0, 0, "");
    col_info.set_type(FlexType::Column);
    col_info.set_pad(6);
    let mut transfer_process = Button::new(0, 0, 0, 0, "Transfer_process");
    let mut delay_48hr = Button::new(0, 0, 0, 0, "48Hr_delay");
    let mut ask_acc = Button::new(0, 0, 0, 0, "Ask_account");
    let mut large_amt = Button::new(0, 0, 0, 0, "Large_amount");
    let mut kpay_limit = Button::new(0, 0, 0, 0, "Kpay_limit");
    let mut dubai_location = Button::new(0, 0, 0, 0, "Dubai_Location");
    col_info.end();

    let mut col_docs = Flex::new(0, 0, 0, 0, "");
    col_docs.set_type(FlexType::Column);
    col_docs.set_pad(6);
    let mut ticket = Button::new(0, 0, 0, 0, "Ticket");
    let mut passport = Button::new(0, 0, 0, 0, "Passport");
    let mut office = Button::new(0, 0, 0, 0, "Office");
    let mut food = Button::new(0, 0, 0, 0, "Food");
    col_docs.end();

    button_grid.end();
    // button_grid has no fixed size on root, so it stretches to absorb any resize

    for b in [
        &mut type_rates, &mut aed_mmk, &mut mmk_aed, &mut one_aed,
        &mut current_aed_acc, &mut current_oman_acc, &mut dubai_kpay_acc,
        &mut ygn_kpay_acc, &mut ygn_wave_acc, &mut ygn_bank_acc,
        &mut transfer_process, &mut delay_48hr, &mut ask_acc, &mut large_amt,
        &mut kpay_limit, &mut dubai_location, &mut ticket, &mut passport,
        &mut office, &mut food,
    ] {
        style_button(b);
    }

    //------------------------------------------------------------------------------------------
    // Footer: calculator inputs/buttons + exit
    let mut footer = Flex::new(0, 0, 0, 0, "");
    footer.set_type(FlexType::Column);
    footer.set_pad(6);

    let mut calc_inputs_row = Flex::new(0, 0, 0, 0, "");
    calc_inputs_row.set_type(FlexType::Row);
    let mut aed_field = Flex::new(0, 0, 0, 0, "");
    aed_field.set_type(FlexType::Row);
    let aed_field_label = Frame::new(0, 0, 0, 0, "AED");
    aed_field.fixed(&aed_field_label, 40);
    let mut input_aed = Input::new(0, 0, 0, 0, "");
    aed_field.end();

    let mut mmk_field = Flex::new(0, 0, 0, 0, "");
    mmk_field.set_type(FlexType::Row);
    let mmk_field_label = Frame::new(0, 0, 0, 0, "MMK");
    mmk_field.fixed(&mmk_field_label, 40);
    let mut input_mmk = Input::new(0, 0, 0, 0, "");
    mmk_field.end();
    style_input(&mut input_aed); style_input(&mut input_mmk);
    let input_aed_clone = input_aed.clone();
    let input_mmk_clone = input_mmk.clone();
    calc_inputs_row.end();
    footer.fixed(&calc_inputs_row, 30);

    let mut calc_buttons_row = Flex::new(0, 0, 0, 0, "");
    calc_buttons_row.set_type(FlexType::Row);
    let mut aed_mmk_k_aed = Button::new(0, 0, 0, 0, "MMK (aed->mmk)");
    let mut mmk_aed_k_aed = Button::new(0, 0, 0, 0, "MMK (mmk->aed)");
    let mut aed_mmk_k_mmk = Button::new(0, 0, 0, 0, "AED (aed->mmk)");
    let mut mmk_aed_k_mmk = Button::new(0, 0, 0, 0, "AED (mmk->aed)");
    calc_buttons_row.end();
    footer.fixed(&calc_buttons_row, 30);

    for b in [&mut aed_mmk_k_aed, &mut mmk_aed_k_aed, &mut aed_mmk_k_mmk, &mut mmk_aed_k_mmk] {
        style_button(b);
    }

    let mut exit_row = Flex::new(0, 0, 0, 0, "");
    exit_row.set_type(FlexType::Row);
    let mut _exit_spacer = Frame::new(0, 0, 0, 0, ""); // pushes Exit to the right
    let mut exit_ = Button::new(0, 0, 0, 0, "Exit");
    style_button(&mut exit_);
    exit_row.fixed(&exit_, 90);
    exit_row.end();
    footer.fixed(&exit_row, 30);

    footer.end();
    root.fixed(&footer, 110);

    root.end();
    wind.resizable(&root);
    //==============================================================================================
    //exit function
    exit_.set_callback(move |_| {exit(0)});
    //==============================================================================================
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
    //rate sending part
    let rates_for_aed_mmk = Rc::clone(&final_rates);
    aed_mmk.set_callback(move |_| {
        let i = rates_for_aed_mmk.borrow();
        wait(3.0);
        keyboard::typewrite("မြန်မာပြည်ငွေလွှဲ 100,000 MMK ကို");next_line();
        keyboard::typewrite(&format!("{} AED (Bank account transfer/Kpay/Wave/True Money)", i[0]));next_line();
        keyboard::typewrite(&format!("{} AED (Cash out at Ygn Office/Cash Home Delivery/ ဘဏ်ထုတ်", i[1]));next_line();
        keyboard::typewrite("ပေးရပါမယ် − ငွေစျေးပြောင်းနိုင်ပါတယ်");next_line();
        keyboard::typewrite("မှတ်ပုံတင်ဖြင့်လွှဲသည်ဖြစ်စေ နယ်အကောင့်များသို့ထည့်သည်ဖြစ်စေ ကျသင့်မည့် ဘဏ် charges များကို customer ဘက်မှသာကျခံပေးရပါမည်");next_line();
        keyboard::typewrite("အိမ်ပို့ငွေများအားလုံး 4-5 ရက်အထိကြာနိုင်ပါတယ်");next_line();
        keyboard::typewrite("ငွေမလွှဲခင်တိုင်းအကောင့်ပြန် confirmပြီးမှလွှဲပေးပါနော်");
        keyboard::key_tap(Vk::Enter);
    });
    let rates_for_mmk_aed = Rc::clone(&final_rates);
    mmk_aed.set_callback(move |_| {
        let i = rates_for_mmk_aed.borrow()[2];
        wait(3.0);
        keyboard::typewrite(&format!("မြန်မာပြည်ကနေငွေလွဲမယ်ဆိုရင် တစ်သိန်းကို {} AED ရပါမယ် စျေးပြောင်းနိုင်ပါတယ်",i));next_line();
        keyboard::typewrite(r"AED ချက်ချင်းပြန်လိုတယ်ဆိုရုံးကို လာထုတ်ပေးမှရပါမယ် အကောင့်ထဲကို လွှဲပေးရမယ်ဆို up to 48 hr အထိကြာနိုင်ပါတယ်");
        keyboard::key_tap(Vk::Enter);
    });
    let rates_for_one_aed = Rc::clone(&final_rates);
    one_aed.set_callback(move |_| {
        let i = rates_for_one_aed.borrow();
        wait(3.0);
        keyboard::typewrite("မြန်မာပြည်ငွေလွှဲ 1 AED ကို");next_line();
        keyboard::typewrite(&format!("{} MMK (Bank account transfer/Kpay/Wave/True Money)", (100000.0/i[0]).round() as i32));next_line();
        keyboard::typewrite(&format!("{} MMK (Cash out at Ygn Office/Cash Home Delivery/ ဘဏ်ထုတ်", (100000.0/i[1]).round() as i32));next_line();
        keyboard::typewrite("ငွေစျေးပြောင်းနိုင်ပါတယ်- *ငွေပြန်လွှဲပေးတဲ့အခါမှာ 1AED နဲ့တွက်ပြီးမလွှဲပေးပါဘူးနော်*");next_line();
        keyboard::typewrite("မှတ်ပုံတင်ဖြင့်လွှဲသည်ဖြစ်စေ နယ်အကောင့်များသို့ထည့်သည်ဖြစ်စေ ကျသင့်မည့် ဘဏ် charges များကို customer ဘက်မှသာကျခံပေးရပါမည်");next_line();
        keyboard::typewrite("ငွေမလွှဲခင်တိုင်းအကောင့်ပြန် confirmပြီးမှလွှဲပေးပါနော်");
        keyboard::key_tap(Vk::Enter);
    });
    //==============================================================================================
    //bottom calculation part
    let r_aed_mmk_k_aed = Rc::clone(&final_rates);
    aed_mmk_k_aed.set_callback(move |_| {
        let rate = r_aed_mmk_k_aed.borrow();
        let aed_amount = input_aed.value();
        match aed_amount.parse::<f64>(){
            Ok(aed_amount)=> {
                let r1 = ((aed_amount / (rate[0] / 100.0)) as i32) * 1000;
                let r2 = ((aed_amount / (rate[1] / 100.0)) as i32) * 1000;
                wait(3.0);
                keyboard::typewrite(&format!("{:.2} AED ကို", aed_amount));next_line();
                keyboard::typewrite(&format!("{} MMK (Bank account transfer/Kpay/Wave/True Money)", r1.to_formatted_string(&Locale::en)));next_line();
                keyboard::typewrite(&format!("{} MMK (Cash out at Ygn Office/Cash Home Delivery/ ဘဏ်ထုတ်)", r2.to_formatted_string(&Locale::en)));next_line();
                keyboard::typewrite("ရပါမယ်");keyboard::key_tap(Vk::Enter);
            }
            Err(_)=>{}
        }
    });
    let r_aed_mmk_k_mmk = Rc::clone(&final_rates);
    aed_mmk_k_mmk.set_callback(move |_| {
        let rate = r_aed_mmk_k_mmk.borrow();
        let mmk_amount = input_mmk.value();
        match mmk_amount.parse::<f64>(){
            Ok(mmk_amount)=> {
                let m1 = (mmk_amount * 100000.0) as i32;
                wait(3.0);
                keyboard::typewrite(&format!("{} MMK ကို", m1.to_formatted_string(&Locale::en)));next_line();
                keyboard::typewrite(&format!("{:.2} AED (Bank account transfer/Kpay/Wave/True Money)", mmk_amount*rate[0]));next_line();
                keyboard::typewrite(&format!("{:.2} AED (Cash out at Ygn Office/Cash Home Delivery/ ဘဏ်ထုတ်)", mmk_amount*rate[1]));next_line();
                keyboard::typewrite("ပေးရပါမယ်");keyboard::key_tap(Vk::Enter);
            }
            Err(_)=>{}
        }
    });
    let r_mmk_aed_k_aed = Rc::clone(&final_rates);
    mmk_aed_k_aed.set_callback(move |_| {
        let rate = r_mmk_aed_k_aed.borrow()[2];
        let aed_amount = input_aed_clone.value();
        match aed_amount.parse::<f64>(){
            Ok(aed_amount)=> {
                let r1 = ((aed_amount / (rate/100.0)) as i32) *1000;
                wait(3.0);
                keyboard::typewrite(&format!("{} AED ကို {} MMK ပေးရပါမယ်",aed_amount, r1.to_formatted_string(&Locale::en)));keyboard::key_tap(Vk::Enter);
            }
            Err(_)=>{}
        }
    });
    let r_mmk_aed_k_mmk = Rc::clone(&final_rates);
    mmk_aed_k_mmk.set_callback(move |_| {
        let rate = r_mmk_aed_k_mmk.borrow()[2];
        let mmk_amount = input_mmk_clone.value();
        match mmk_amount.parse::<f64>(){
            Ok(mmk_amount)=> {
                let m1 = (mmk_amount * 100000.0) as i32;
                wait(3.0);
                keyboard::typewrite(&format!("{} MMK ကို {} AED ရပါမယ်", m1.to_formatted_string(&Locale::en) , mmk_amount*rate));keyboard::key_tap(Vk::Enter);
            },
            Err(_)=>{}
        }
    });
    //==============================================================================================
    //rate updating and error checking
    let mut popup_for_show = rate_popup.clone();
    type_rates.set_callback(move |_|{
        popup_for_show.show()
    });
    let mut popup_for_hide = rate_popup.clone();
    confirm_rates.set_callback(move |_| {
        let got_rates = get_rates(&aed1.value(), &aed2.value(), &mmk.value());
        if got_rates.contains(&0.0) {
            let mut warn = Window::new(300, 150, 300, 130, "Warning");
            style_window(&mut warn);
            let mut warn_root = Flex::new(0, 0, 300, 130, "");
            warn_root.set_type(FlexType::Column);
            warn_root.set_margin(15);
            warn_root.set_pad(10);
            let mut warn_msg = Frame::new(0, 0, 0, 0, "Please enter valid numbers and try again!");
            warn_msg.set_label_color(Color::from_rgb(255, 99, 99));
            warn_msg.set_align(Align::Wrap | Align::Center);
            let mut ok = Button::new(0, 0, 0, 0, "Ok");
            style_button(&mut ok);
            warn_root.fixed(&ok, 30);
            warn_root.end();
            warn.resizable(&warn_root);
            warn.make_modal(true);
            warn.end();
            warn.show();
            ok.set_callback(move |_| { warn.hide() });
        }
        else{
            frame1.set_label(&format!("{}", got_rates[0]));
            frame2.set_label(&format!("{}", got_rates[1]));
            frame3.set_label(&format!("{}", got_rates[2]));
            *rates_to_update.borrow_mut() = got_rates;
            popup_for_hide.hide()
        }
    });
    //==============================================================================================
    wind.end();
    wind.show();
    rate_popup.show();
    app.run().unwrap();
}