use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_NAME_FORMAT};
use windows::Win32::Media::Audio::{MMDeviceEnumerator, IMMDeviceEnumerator, eRender, eConsole, IAudioSessionManager2, IAudioSessionControl2, AudioSessionStateActive};
//==================================================================================================
fn send_play_pause_to_spotify(){
    use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput,INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VK_MEDIA_PLAY_PAUSE};
    let key = |flags: KEYBD_EVENT_FLAGS| INPUT{
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki:KEYBDINPUT{
                wVk: VK_MEDIA_PLAY_PAUSE,wScan:0,dwFlags:flags,time:0,dwExtraInfo:0
            }
        }
    };
    let input = [key(KEYBD_EVENT_FLAGS(0)),key(KEYEVENTF_KEYUP)];
    unsafe {
        SendInput(&input, size_of::<INPUT>() as i32 );
    }
}
unsafe fn audible_exes(manager:&IAudioSessionManager2)-> windows::core::Result<Vec<String>>{
    let sessions = manager.GetSessionEnumerator()?;
    let count = sessions.GetCount()?;
    let mut out = Vec::new();
    for i in 0..count{
        let Ok(ctrl) = sessions.GetSession(i) else {continue};
        match ctrl.GetState(){
            Ok(s) if s == AudioSessionStateActive=>{},
            _ => continue,
        }
        use windows::core::Interface; //.cast uses this import
        let Ok(meter) = ctrl.cast::<windows::Win32::Media::Audio::Endpoints::IAudioMeterInformation>() else {continue};
        if meter.GetPeakValue().unwrap_or(0.0) <=PEAK_THRESHOLD{continue;}
        let Ok(ctrl2) = ctrl.cast::<IAudioSessionControl2>() else {continue};
        let pid = ctrl2.GetProcessId().unwrap_or(0);
        if pid == 0 {continue}
        let name = exe_name(pid).unwrap_or_else(||format!("unknown({pid})"));
        out.push(name.to_lowercase());
    }
    Ok(out)
}
fn exe_name(pid:u32)->Option<String>{
    unsafe{
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION,false, pid).ok()?;
        let mut buf = vec![0u16;1024];
        let mut size = buf.len() as u32;
        let res = QueryFullProcessImageNameW(handle,PROCESS_NAME_FORMAT(0),windows::core::PWSTR(buf.as_mut_ptr()), &mut size);
        let _ = windows::Win32::Foundation::CloseHandle(handle);
        res.ok()?;
        let path = String::from_utf16_lossy(&buf[..size as usize]);
        std::path::Path::new(&path).file_name().map(|n|n.to_string_lossy().into_owned())
    }
}
fn monitor() {
    unsafe {
        windows::Win32::System::Com::CoInitializeEx(None, windows::Win32::System::Com::COINIT_MULTITHREADED)
            .ok()
            .expect("CoInitializeEx failed");

        let enumerator: IMMDeviceEnumerator =
            windows::Win32::System::Com::CoCreateInstance(&MMDeviceEnumerator, None, windows::Win32::System::Com::CLSCTX_ALL).expect("CoCreateInstance failed");
        let device = enumerator
            .GetDefaultAudioEndpoint(eRender, eConsole)
            .expect("GetDefaultAudioEndpoint failed");
        let manager: IAudioSessionManager2 = device
            .Activate::<IAudioSessionManager2>(windows::Win32::System::Com::CLSCTX_ALL, None)
            .expect("Activate failed");

        let mut last_heard: std::collections::HashMap<String, std::time::Instant> = std::collections::HashMap::new();
        let mut paused_by_us = false;
        let mut quiet_since: Option<std::time::Instant> = None;

        loop {
            std::thread::sleep(POLL);
            let now = std::time::Instant::now();

            if let Ok(list) = audible_exes(&manager) {
                for name in list {
                    last_heard.insert(name, now);
                }
            }
            last_heard.retain(|_, t| now.duration_since(*t) < std::time::Duration::from_secs(30));

            let recent = |t: &std::time::Instant| now.duration_since(*t) < HOLD;
            let spotify_active = last_heard.get(SPOTIFY).map_or(false, recent);
            let others_active = last_heard
                .iter()
                .any(|(name, t)| name != SPOTIFY && recent(t));

            if !paused_by_us {
                if spotify_active && others_active {
                    println!("Other audio detected -> pausing Spotify");
                    send_play_pause_to_spotify();
                    paused_by_us = true;
                    quiet_since = None;
                }
            } else if others_active {
                quiet_since = None;
            } else {
                let since = *quiet_since.get_or_insert(now);
                if now.duration_since(since) >= RESUME_DELAY {
                    println!("Others went quiet -> resuming Spotify");
                    send_play_pause_to_spotify();
                    paused_by_us = false;
                    quiet_since = None;
                }
            }
        }
    }
}
//==================================================================================================
pub async fn listen(){
    let _ = tokio::task::spawn_blocking(monitor).await;
}
//==================================================================================================
const PEAK_THRESHOLD:f32 = 0.0001;
const POLL:std::time::Duration = std::time::Duration::from_millis(250);
const HOLD:std::time::Duration = std::time::Duration::from_millis(1500);
const RESUME_DELAY:std::time::Duration = std::time::Duration::from_secs(1);
const SPOTIFY:&str = "spotify.exe";