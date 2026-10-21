//Todo
/*
1. threading multiple functions is needed to loop different type of reminders
2. store reminders in a file so they are permanent (storage system? read/write)
3. timely reminders, daily, monthly, yearly etc... (repeating feature?)
4. audio attaching to reminders?
5. GUI??? (using console before gui)
6. writing and deleting reminders ----> remove how in console? (print them with a number and delete with number?) - gui first?
*/
//imports
use colored::*;
use powershell_script::PsScriptBuilder;
use rodio::{DeviceSinkBuilder,Decoder};
use windows_sys::core::BOOL;
use chrono::{Datelike, Local, Timelike};
use rsautogui::{keyboard, keyboard::Vk};
use std::{time::{Duration},thread,path::PathBuf, fs::File};
use windows_sys::Win32::Foundation::{HWND, LPARAM, TRUE};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::BlockInput;
use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SetForegroundWindow, SW_MAXIMIZE, IsWindowVisible, GetWindowTextLengthW, GetWindowTextW, EnumWindows};
//==================================================================================================
//general/helper functions
fn pause_spotify(){
    let ps = PsScriptBuilder::new().no_profile(true).non_interactive(true).build();
    let script = r#"
        $shell = New-Object -ComObject WScript.Shell
        $shell.SendKeys([char]179)
    "#;
    let _ = ps.run(script);
}
fn find_file(filename:&str)->Option<PathBuf>{
    for folder in walkdir::WalkDir::new("C:\\GIT").into_iter() {
        let Ok(file) = folder else {continue};
        if file.file_name() == filename && file.file_type().is_file() {
            return Some(file.path().to_path_buf());
        }
    }
    None
}
fn play_audio(audio_file:&str){
    let f = find_file(audio_file).unwrap();
    println!("{:?}",f);
    let file = File::open(f).unwrap();
    let handle = DeviceSinkBuilder::open_default_sink()
        .expect("open default audio stream");
    let source = Decoder::try_from(file).unwrap();
    handle.mixer().add(source);
    std::thread::sleep(std::time::Duration::from_secs(5));
}
fn find_windows(){
    unsafe extern "system" fn find(win: HWND, _lparam: LPARAM) -> BOOL {
        unsafe{
        if IsWindowVisible(win) != 0{
            let length = GetWindowTextLengthW(win);
            if length > 0 {
                let mut buf = vec![0u16; (length + 1) as usize];
                let actual_len = GetWindowTextW(win, buf.as_mut_ptr(), buf.len() as i32);
                if actual_len > 0 {
                    let title = String::from_utf16_lossy(&buf[..actual_len as usize]);
                    if !title.trim().is_empty() {
                        if title.to_lowercase().contains("opera") {
                            println!("Found opera!");
                            BlockInput(1);
                            ShowWindow(win, SW_MAXIMIZE);
                            SetForegroundWindow(win);
                            return TRUE;
                        }
                    }
                }
                BlockInput(0);
            }
        }
        TRUE
        }
    }
    unsafe {
        EnumWindows(Some(find), 0);
    }
}//finds the active windows
fn wait(seconds:f64){thread::sleep(Duration::from_secs(seconds as u64));}//simplifying wait for multiple uses
//==================================================================================================
pub fn remind_messenger(){
    loop{
        play_audio("alert.mp3");
        find_windows();
        wait(1.0);
        keyboard::key_down(Vk::Control);
        keyboard::key_tap(Vk::_2);
        keyboard::key_up(Vk::Control);
        thread::sleep(Duration::from_mins(10));
    }
}
pub fn remind_daily_tasks(){
    loop {
        thread::sleep(Duration::from_secs(10));
        let now = Local::now();
        let h = now.hour();
        let m = now.minute();
        let s = now.second();
        if h == 23 && m == 50 {
            println!("Reminder for printing bank statements!");
            pause_spotify();
            play_audio("alert.mp3");
            return;
        }
        if h == 3 && m == 30 {
            println!("Reminder for filling in Sheet11!");
            pause_spotify();
            play_audio("alert.mp3");
            return;
        }
    }
}
pub fn remind_day_specific_tasks(){
    let now = Local::now();
    let d = now.date().day();
    println!("{}",d);
    if d > 25 && d < 30 {
        loop{
            thread::sleep(Duration::from_mins(10));
            println!("{}","Reminder to do finger print!".red());
        }
    }
}