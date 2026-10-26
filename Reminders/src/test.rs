use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Local};
use directories::ProjectDirs;
use serde::{Serialize,Deserialize};
#[derive(Serialize, Deserialize, Debug)]
struct Task{done:bool, id:DateTime<Local>, task:String}
type JsonStorage = Vec<Task>;
fn load_json()-> JsonStorage {
    let path = storage();
    if path.exists(){
        let contents = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&contents).unwrap_or_default()
    }
    else { Vec::new() }
}
fn storage()->PathBuf{
    if let Some(proj_dirs) = ProjectDirs::from("com", "Nessie", "Reminder"){
        let data_dir = proj_dirs.data_dir();
        fs::create_dir_all(data_dir).expect("Failed to create app data directory!");
        data_dir.join("storage.json")
    }else{
        PathBuf::from("storage.json")
    }
}
pub fn test(){
    let all_tasks = load_json();
    for task in all_tasks.iter(){
        println!("{:?}",task);
        println!("{:?}",task.task);
    }
}