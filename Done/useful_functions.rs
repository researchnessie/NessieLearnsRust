use std::{fs, io};

fn find_file(filename:&str) ->Option<PathBuf>{
    let unknown_entry_folder = dirs::download_dir()?; //use this for user specific folders
    let known_entry_folder = r"C:\";
    for f in WalkDir::new(unknown_entry_folder).into_iter().flatten() {
        if f.file_name() == filename && f.file_type().is_file() {
            return Some(f.path().to_path_buf())
        }
    }
    None
}
fn read_and_write(input:String)->Result<(), io::Error> {
    let file = find_file("output.txt");
    println!("{:?}",file);
    if file == None {
        fs::write("assets/output.txt", input)?; //give your own filepath
    }
    else{fs::write(file.unwrap(), input)?;}
    Ok(())
}