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