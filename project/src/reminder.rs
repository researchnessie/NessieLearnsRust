use std::{time::Duration, thread};
use espeak_ng::{text_to_ipa, EspeakNg, text_to_pcm};
use rsautogui::{keyboard, keyboard::Vk};
use chrono::{Local, Timelike};

pub fn remind(){
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_mins(10));
            remind_messenger();
        }
    });
    loop{
        thread::sleep(Duration::from_secs(5));
        remind_sheet11()
    }
}
fn remind_messenger(){println!("Remind Messenger");}
fn remind_sheet11(){println!("Remind Sheet11");}

pub fn test(){
    match text_to_ipa("en","hello world") {
        Ok(ipa) => {println!("{:?}",ipa);}
        Err(e) => {println!("{:?}",e);}
    }
}
pub fn test_audio(){
    match text_to_pcm("en", "hello world!"){
        Ok((samples, sample_rate)) =>{
            println!("{},{}",samples.len(), sample_rate);
        }
        Err(e) => {println!("{:?}",e);}
    }
}