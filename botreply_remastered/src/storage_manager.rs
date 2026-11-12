use walkdir::WalkDir;
use whatsapp_rust::serde_json;
use serde::{Serialize, Deserialize};
use directories::{ProjectDirs, UserDirs};
use std::{fs,path::PathBuf,process::exit};
//==================================================================================================
/*
These struct,types and function below act as a way to store all the customer's interacted throughout the day to be updated in Reminders App and followed up
the struct Task is made to match the data requirements of the Reminders App, so is the type JsonStorage
user_id_collector() look for the file to load the existing data, if there is no file, it makes one
read and write function does exactly what they say
*/
#[derive(Serialize, Deserialize, Clone)]
//this struct is created according to the needs of the reminder app's structure
//for more info of the workings of reminder app, read the code of the reminder app
pub struct Task{pub task:String, pub hour:u32, pub min:u32, pub done:bool}
type JsonStorage = Vec<Task>;
//This function looks for user_storage.json inside current user's data folder
fn user_id_collector()->PathBuf{
    if let Some(target_folder) = ProjectDirs::from("com", "AutoReply", "UserData") {
        let users_data_folder = target_folder.data_dir();
        fs::create_dir_all(users_data_folder).expect("Failed to create app users folder!");
        users_data_folder.join("user_storage.json")
    }
    else{
        PathBuf::from("user_storage.json")
    }
    //if user already has AutoReply/UserData inside his data folder, make user_storage.json, if not crate the entire path then make the user_storage.json
}
//This function is the read function to the previous user_id_collector()
pub fn read_userid()->JsonStorage{
    let path = user_id_collector();
    if path.exists(){
        let contents = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&contents).unwrap_or_default()
    }
    else {Vec::new()}
    //if the user_id_collector() can give the collection JSON then make a JsonStorage(which is a vec of the struct Task) out of it, else create a new Vector
}
//This function is the write function to the previous user_id_collector()
pub fn write_userid(data_to_write:JsonStorage){
    let path = user_id_collector();
    let temp_path = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(&data_to_write).expect("Failed to serialize json!");
    fs::write(&temp_path, json).expect("Failed to write json!");
    fs::rename(temp_path, path).expect("Failed to rename json!");
    //take the data_to_write, and write it in the temporary file .json.tmp then rename it back to just .json
}
//==================================================================================================
//This function loads and stores login session data in the form of .db, and in case of file missing it creates a new one and load that instead
//This function will be used together with the auto bot login in the main code
pub fn login_data_storage()->String{
    let login_db_path = if let Some(proj_dirs) = ProjectDirs::from("com", "AutoReply", "WhatsAppBot") {
        let data_dir = proj_dirs.data_dir();
        fs::create_dir_all(data_dir).expect("Failed to create app users folder!");
        data_dir.join("whatsapp_session.db")
    }
    else{
        PathBuf::from("whatsapp_session.db")
    };
    login_db_path.to_str().unwrap().to_string()
}
//==================================================================================================
//These two functions collects user messages to be trained with onnx later
//These doesn't have dynamic pathing or missing problems since I know the file exists and these are also temporary collection
type TrainingDataStorage = Vec<String>;
pub fn load_user_messages()->TrainingDataStorage{
    let path = r"C:\GIT\training_data.json";
    let contents = fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&contents).unwrap_or_default()
}
pub fn save_user_messages(data_to_write:TrainingDataStorage){
    let path = r"C:\GIT\training_data.json";
    let tmp_path = format!("{}.tmp", &path);
    let json = serde_json::to_string_pretty(&data_to_write).expect("Failed to serialize json!");
    fs::write(&tmp_path, json).expect("Failed to write json!");
    fs::rename(&tmp_path, &path).expect("Failed to rename tmp file!");
}
//==================================================================================================
//This function loads and prepare the input picture and in case of failer it terminates the program
//This is made this way to prevent the bot from working with assets missing
pub fn load_picture(filename:&str)->Vec<u8>{
    let user_folder = UserDirs::new().unwrap();
    let download_folder = user_folder.download_dir().unwrap();
    for entry in WalkDir::new(download_folder).into_iter().flatten() {
        if entry.file_name() == filename && entry.file_type().is_file() {
            if let Ok(data) = fs::read(entry.path()){
                return data;
            }
        }
    }
    println!("{} is missing in the download folder!",filename);exit(1);
}
//==================================================================================================