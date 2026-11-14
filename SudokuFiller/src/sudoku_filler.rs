use rsautogui::{keyboard,keyboard::Vk};
use std::{thread,time::Duration};
//==================================================================================================
//This function is created to shorten the same pressing of 1-9 over and over
fn press_buttons(){
    let digits = [Vk::Numpad1, Vk::Numpad2, Vk::Numpad3, Vk::Numpad4, Vk::Numpad5, Vk::Numpad6, Vk::Numpad7, Vk::Numpad8, Vk::Numpad9];
    for digit in digits {keyboard::key_tap(digit);}
}
//There is a 5 seconds delay before it starts filling the empty cells to allow users to navigate to the starting cell
//The program will fill cell by cell in serpentine pattern (down, then up, moving to the right)
pub fn fill() {
    thread::sleep(Duration::from_secs(5));
    let mut mode = Vk::DownArrow;
    let mut column_count:u32 = 1;
    while column_count < 10 {//this while loop moves to the right each time a column is filled
        column_count += 1;
        let mut cell_count:u32= 1;
        while cell_count < 10 {//this while loop fills in an entire column, cell by cell depending on current mode
            press_buttons();
            keyboard::key_tap(mode);
            cell_count += 1;
        }
        keyboard::key_tap(Vk::RightArrow);//move to the right column once this column is done
        mode = if mode == Vk::DownArrow { Vk::UpArrow } else { Vk::DownArrow }
    }
}