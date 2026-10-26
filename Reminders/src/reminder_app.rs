use eframe;
use directories::ProjectDirs;
use chrono::{Local, DateTime, Timelike, Datelike};
use serde::{Serialize,Deserialize};
use powershell_script::PsScriptBuilder;
use rodio::{DeviceSinkBuilder,Decoder};
use rsautogui::{keyboard, keyboard::Vk};
use std::{fs,path::PathBuf,process::exit,thread,time::Duration};

use windows_sys::core::BOOL;
use windows_sys::Win32::Foundation::{HWND, LPARAM, TRUE};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::BlockInput;
use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SetForegroundWindow, SW_MAXIMIZE, IsWindowVisible, GetWindowTextLengthW, GetWindowTextW, EnumWindows};
//==================================================================================================
pub fn run_main()->eframe::Result{
    thread::spawn(move || {remind_messenger()});
    thread::spawn(move || {remind_daily_tasks()});
    let mut options = eframe::NativeOptions::default();
    options.viewport= egui::ViewportBuilder::default().with_icon(load_icon());
    eframe::run_native(
        "Reminders and todo lists", options,
    Box::new(|app| {
        app.egui_ctx.set_visuals(egui::Visuals::dark());
        app.egui_ctx.set_pixels_per_point(1.5);
        Ok(Box::new(Reminder::default()))
    })
    )
}
//==================================================================================================
//Storage part
fn storage()->PathBuf{
    if let Some(proj_dirs) = ProjectDirs::from("com", "Nessie", "Reminder"){
        let data_dir = proj_dirs.data_dir();
        fs::create_dir_all(data_dir).expect("Failed to create app data directory!");
        data_dir.join("storage.json")
    }else{
        PathBuf::from("storage.json")
    }
}
fn load_json()-> JsonStorage {
    let path = storage();
    if path.exists(){
        let contents = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&contents).unwrap_or_default()
    }
    else { Vec::new() }
}
fn save_json(data: JsonStorage){
    let path = storage();
    let json = serde_json::to_string_pretty(&data).expect("Failed to serialize json!");
    fs::write(&path, json).expect("Failed to write json!");
}
//==================================================================================================
struct Reminder{
    task:JsonStorage,
    add_task:String,
    missing_files:bool
}
impl Reminder{
    fn default()->Self {
        //check files before we start
        let mut b = false;
        if find_file("alert.mp3") == None{
            b = true
        }
        //==========================================================================================
        let mut all_tasks = load_json();
        //add daily/monthly reminders
        let now = Local::now();
        //fingerprint
        if 26 <= now.day() && now.day() <= 29 {all_tasks.push(Task{id:now,task:String::from("Fingerprint list"),done:false})}//leaving it intentionally so I know it is pending for longer
        let mut checker:Vec<String> = vec![];
        for t in all_tasks.iter(){
            checker.push(t.task.clone());
        }
        if !checker.contains(&"Print all the bank statements!".to_string()){
            all_tasks.push(Task{id:now,task:String::from("Print all the bank statements!"),done:false});
        }
        if !checker.contains(&"Fill in sheet11 completely!".to_string()){
            all_tasks.push(Task{id:now,task:String::from("Fill in sheet11 completely!"),done:false});
        }
        save_json(all_tasks.clone());
        Self{task: all_tasks,add_task:String::new(),missing_files:b}
    }
}
impl eframe::App for Reminder{
    fn ui(&mut self, ui: &mut egui::Ui,_frame:&mut eframe::Frame){
        if self.missing_files ==true{
            egui::Window::new("Error!").show(ui, |ui|{
                ui.add(egui::Label::new("There are files missing for the program to function!").wrap());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),
                               |ui|{
                                   if ui.button("Exit").clicked(){exit(0);}}
                );
            });
        }
        egui::CentralPanel::default().show(ui,|ui|{
            ui.add_space(10.0);
            //head area
            ui.horizontal(|ui|{
                ui.heading("Reminders");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),
                               |ui|{
                                   if ui.button("Exit").clicked(){exit(0);}}
                );
            });
            ui.separator();ui.add_space(10.0);
            //======================================================================================
            ui.label("All pending tasks!").on_hover_text("Here listed are all the tasks pending to be done!");
            ui.separator();
            //listing tasks
            let mut changed = false;
            egui::ScrollArea::vertical().max_height(ui.available_height()-80.0).max_width(ui.available_width()-50.0)
                .show(ui, |ui| {
                for task in &mut self.task{
                    ui.horizontal(|ui|{
                        if ui.checkbox(&mut task.done, "").changed(){
                            changed = true;
                        }
                        if ui.add(egui::Label::new(format!("{} - {}", task.id.format("%H:%M"), task.task)).wrap()).clicked(){
                            task.done = !task.done;
                            changed = true;
                        }
                    });
                }
            });
            //removing tasks
            if changed {
                self.task.retain(|t|!t.done);
                save_json(self.task.clone());
            }
            ui.separator();
            //adding tasks
            ui.horizontal(|ui|{
                ui.add(egui::TextEdit::singleline(&mut self.add_task).desired_width(ui.available_width()-90.0));
                let ready = ui.button("Add Task").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter));
                if  ready && !self.add_task.is_empty(){
                    self.task.push(Task{
                        id:Local::now(),
                        task:self.add_task.clone(),
                        done:false,
                    });
                    self.add_task.clear();
                    save_json(self.task.clone());
                }
            });
        });
    }
}
//==================================================================================================
#[derive(Serialize, Deserialize, Clone)]
struct Task{done:bool, id:DateTime<Local>, task:String}
type JsonStorage = Vec<Task>;
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
    let f = find_file(audio_file);
    let file = fs::File::open(f.unwrap()).unwrap();
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
                                BlockInput(1);
                                ShowWindow(win, SW_MAXIMIZE);
                                SetForegroundWindow(win);
                                BlockInput(0);
                                return TRUE;
                            }
                        }
                    }
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
fn load_icon()->egui::IconData{
    let image = image::load_from_memory(include_bytes!("../assets/icon.png")).expect("Failed to load icon").to_rgba8();
    let rgba = image.clone().into_raw();
    let (width, height) = image.dimensions();
    egui::IconData {rgba, width, height}

}
//==================================================================================================
//loops
fn remind_messenger(){
    loop{
        play_audio("alert.mp3");
        find_windows();
        wait(3.0);
        keyboard::key_down(Vk::Control);
        keyboard::key_tap(Vk::_2);
        keyboard::key_up(Vk::Control);
        thread::sleep(Duration::from_mins(10));
    }
}
fn remind_daily_tasks(){
    loop {
        thread::sleep(Duration::from_secs(10));
        let now = Local::now();
        let h = now.hour();
        let m = now.minute();
        if h == 23 && m == 50 {
            pause_spotify();
            play_audio("alert.mp3");
            return;
        }
        if h == 3 && m == 30 {
            pause_spotify();
            play_audio("alert.mp3");
            return;
        }
    }
}
