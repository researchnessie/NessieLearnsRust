use rsautogui::{keyboard,keyboard::Vk};
use std::{thread,time::Duration};
//==================================================================================================
fn press_buttons(){
    let digits = [Vk::Numpad1, Vk::Numpad2, Vk::Numpad3, Vk::Numpad4, Vk::Numpad5, Vk::Numpad6, Vk::Numpad7, Vk::Numpad8, Vk::Numpad9];
    for digit in digits {keyboard::key_tap(digit);}
}//simplifying key inputs
pub fn fill() {
    thread::sleep(Duration::from_secs(5));
    let mut mode = Vk::DownArrow;
    let mut lane_count = 0;
    while lane_count <9 {
        lane_count += 1;
        let mut a = 1;
        while a < 10 {
            press_buttons();
            keyboard::key_tap(mode);
            a += 1;
        }
        keyboard::key_tap(Vk::RightArrow);
        if mode == Vk::DownArrow { mode = Vk::UpArrow; } else { mode = Vk::DownArrow; }
    }
}