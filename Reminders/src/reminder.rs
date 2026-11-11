//imports
use eframe;
use dirs;
use directories::ProjectDirs;
use chrono::{Local, Timelike, Datelike};
use serde::{Serialize,Deserialize};
use powershell_script::PsScriptBuilder;
use rodio::{DeviceSinkBuilder,Decoder};
use std::{fs,path::PathBuf,process::exit,thread,time::Duration};
//==================================================================================================
//general/helper functions
fn load_icon()->egui::IconData{
    let image = image::load_from_memory(include_bytes!("../assets/icon.png")).expect("Failed to load icon").to_rgba8();
    let rgba = image.clone().into_raw();
    let (width, height) = image.dimensions();
    egui::IconData {rgba, width, height}
}
fn toggle_spotify(){
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
fn play_audio(audio_file:PathBuf){
    let file = fs::File::open(audio_file).unwrap();
    let handle = DeviceSinkBuilder::open_default_sink()
        .expect("open default audio stream");
    let source = Decoder::try_from(file).unwrap();
    handle.mixer().add(source);
    thread::sleep(Duration::from_secs(5));
}
//==================================================================================================
//general types and structs
#[derive(Serialize, Deserialize, Clone)]
struct Task{task:String, hour:u32, min:u32, done:bool}
type JsonStorage = Vec<Task>;
//==================================================================================================
//Storage part
fn storage()->PathBuf{
    if let Some(proj_dirs) = ProjectDirs::from("com", "Nessie", "Reminder"){
        let data_dir = proj_dirs.data_dir();
        fs::create_dir_all(data_dir).expect("Failed to create app data directory!");
        data_dir.join("reminder_storage.json")
    }else{
        PathBuf::from("reminder_storage.json")
    }
}
fn load_json(path:PathBuf)-> JsonStorage {
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
//loops
fn remind_messenger(x:PathBuf){
    loop{
        play_audio(x.clone());
        thread::sleep(Duration::from_mins(10));
    }
}
//==================================================================================================
struct Reminder{
    task:JsonStorage, add_task:String,
    due_hr:u32, due_min:u32,
    missing_file:bool, file_path:Option<PathBuf>,
    dismissed:Vec<String>,alert:Option<String>,
}
impl Reminder{
    fn default()->Self{
        //check files before we start
        let file_path = find_file("alert.mp3");
        let file_missing = file_path.is_none();
        //load tasks from storage
        let mut all_tasks = load_json(storage());
        //add default tasks
        let now = Local::now();
        if 26 <= now.day() && now.day() <= 29 {all_tasks.push(Task{task:String::from("Fingerprint list"),done:false,hour:11,min:30})}
        let mut checker:Vec<String> = vec![];
        for t in all_tasks.iter(){
            checker.push(t.task.clone());
        }
        if !checker.contains(&"Print all the bank statements!".to_string()){
            all_tasks.push(Task{hour:23,min:55,task:String::from("Print all the bank statements!"),done:false});
        }
        if !checker.contains(&"Fill in sheet11 completely!".to_string()){
            all_tasks.push(Task{hour:3,min:15,task:String::from("Fill in sheet11 completely!"),done:false});
        }
        save_json(all_tasks.clone());
        //update all the data
        Self{
            task:all_tasks, add_task:String::new(),
            due_hr:0, due_min:0,
            missing_file:file_missing, file_path,
            dismissed:Vec::new(), alert:None
        }
    }
}
impl eframe::App for Reminder{
    fn ui(&mut self,ui:&mut egui::Ui,_frame:&mut eframe::Frame){
        ui.ctx().request_repaint_after(Duration::from_secs(10));//keeps the loop active aka doesn't let egui to go to sleep
        if self.missing_file {
            egui::Window::new("Error!").show(ui, |ui|{
                ui.add(egui::Label::new("There are files missing for the program to function!").wrap());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),
                |ui|{if ui.button("Exit").clicked(){exit(0);}}
                );
            });
        }//checks if there are any files missing and ends the program
        egui::CentralPanel::default().show(ui, |ui|{
            //header area and exit button
            ui.add_space(10.0);
            ui.horizontal(|ui|{
                ui.heading("Reminder");
                if ui.button("Exit").clicked(){exit(0)};
                if ui.button("Update").clicked(){
                    for folder in walkdir::WalkDir::new(dirs::data_dir().unwrap()).into_iter(){
                        let Ok(file) = folder else {continue};
                        if file.file_name() == "user_storage.json" && file.file_type().is_file() {
                            let new_data = load_json(file.path().to_path_buf());
                            let mut old_data = load_json(storage());
                            for t in new_data.iter(){
                                old_data.push(t.clone());
                            }
                            self.task = old_data;
                            save_json(self.task.clone());
                            let _ = fs::write(file.path(), "[]");
                        }
                    }
                }
            });
            ui.add_space(10.0);
            ui.separator();
            //======================================================================================
            ui.label("All pending tasks with their recommended execution times!");
            ui.separator();
            //======================================================================================
            //listing tasks
            let mut changed = false;
            egui::ScrollArea::vertical().max_height(ui.available_height()-80.0).max_width(ui.available_width()-50.0)
                .show(ui, |ui| {
                    for task in &mut self.task{
                        ui.horizontal(|ui|{
                            if ui.checkbox(&mut task.done, "").changed(){
                                changed = true;
                            }
                            if ui.add(egui::Label::new(format!("{}:{} - {}", task.hour, task.min, task.task)).wrap()).clicked(){
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
            //======================================================================================
            //adding tasks
            ui.horizontal(|ui|{
                ui.add(egui::DragValue::new(&mut self.due_hr).suffix("H").range(0..=23));
                ui.add(egui::DragValue::new(&mut self.due_min).suffix("M").range(0..=59));
                ui.add(egui::TextEdit::singleline(&mut self.add_task).desired_width(ui.available_width()-200.0));
                let ready = ui.button("Add Task").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter));
                if  ready && !self.add_task.is_empty(){
                    self.task.push(Task{done:false,task:self.add_task.clone(),hour:self.due_hr,min:self.due_min});
                    self.add_task.clear();
                    save_json(self.task.clone());
                }
            });
            //======================================================================================
            //looping part to alert
            if self.alert.is_none(){
                let now = Local::now();
                let now_min = now.hour()*60 + now.minute();
                if let Some(t) = self.task.iter().find(|t| t.hour*60 + t.min == now_min && !self.dismissed.contains(&t.task)) {
                    self.alert = Some(t.task.clone());
                    thread::spawn(toggle_spotify); //no idea why its in a thread, not yet - I think it is to run it only once.
                    if let Some(p) = &self.file_path {play_audio(p.clone())}
                }
            }
            if let Some(name) = self.alert.clone() {
                let (mut snooze, mut ok) = (false, false);
                egui::Window::new("Task Due!").collapsible(false).show(ui, |ui| {
                    ui.label(format!("It's time to {}", name));
                    ui.horizontal(|ui| {
                        if ui.button("Snooze 5 min").clicked() { snooze = true; }
                        if ui.button("Ok").clicked() { ok = true; }
                    });
                });
                if snooze {
                    let later = Local::now() + chrono::Duration::minutes(5);
                    if let Some(t) = self.task.iter_mut().find(|t| t.task == name) {
                        t.hour = later.hour();
                        t.min = later.minute();
                    }
                    save_json(self.task.clone());
                    thread::spawn(toggle_spotify);
                    self.alert = None;
                } else if ok {
                    self.dismissed.push(name);
                    thread::spawn(toggle_spotify);
                    self.alert = None;
                }
            }
        });
    }
}
//==================================================================================================
pub fn start_app()-> eframe::Result{
    let x = find_file("alert.mp3");
    thread::spawn(move || {remind_messenger(x.unwrap())});
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
