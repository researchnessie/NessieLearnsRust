use std::thread;
use std::time::Duration;
// use std::{time::Duration, thread};
// use espeak_ng::{text_to_ipa, EspeakNg, text_to_pcm};
// use rsautogui::{keyboard, keyboard::Vk};
// use chrono::{Local, Timelike};
//
// pub fn remind(){
//     thread::spawn(move || {
//         loop {
//             thread::sleep(Duration::from_mins(10));
//             remind_messenger();
//         }
//     });
//     loop{
//         thread::sleep(Duration::from_secs(5));
//         remind_sheet11()
//     }
// }

pub fn test(){
    loop{
        let now = Local::now();
        let current_hour = now.hour();
        let current_min = now.minute();
        if current_hour == 23 && current_min == 48 {
            loop{println!("Now is the time")}
        }
        thread::sleep(Duration::from_secs(10))
    }
}