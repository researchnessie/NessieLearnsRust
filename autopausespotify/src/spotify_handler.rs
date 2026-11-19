use std::error::Error;
use serde::Deserialize;

//==================================================================================================
#[derive(Deserialize)]
struct SpotifyHandler{
    playing:bool,
}
pub async fn handler() -> Result<(), Box<dyn Error>>{
    println!("Scanning for Spotify local server...");
    let (port, csrf_token, oauth_token) = find_spotify_credentials().await.ok_or("Could not find Spotify access token")?;
    println!("main_handler_got variables");
    //println!("{}",format!("found Spotify token: {}, and its port->{}", csrf_token, port).to_string());
    toggle_spotify_playback(port, &csrf_token, &oauth_token).await?;
    Ok(())
}
async fn find_spotify_credentials()->Option<(u16, String, String)>{
    println!("Finding spotify credentials...");
    let client = reqwest::Client::builder().danger_accept_invalid_certs(true).build().unwrap();
    println!("client made");
    for port in 4000..=4900{
        let url = format!("https://local.spotify.com:{}/remote/status.json",port);
        if let Ok(res) = client.get(&url).header("Origin", "https://open.spotify.com").send().await {
            if res.status().is_success() {
                if let Ok(json) = res.json::<serde_json::Value>().await {
                    if let (Some(csrf),Some(oauth)) = (
                        json.get("csrf_token").and_then(|v|v.as_str()),
                        json.get("oauth_token").and_then(|v|v.as_str()),
                        ){
                        return Some((port, csrf.to_string(),oauth.to_string()));
                    }
                }

            }
        }
    }
    println!("for loop finished and returning none");
    None
}
async fn toggle_spotify_playback(port: u16, csrf_token: &str, oauth_token: &str)->Result<(),Box<dyn Error>> {
    let client = reqwest::Client::builder().danger_accept_invalid_certs(true).build()?;
    let status_url = format!("https://local.spotify.com:{}/remote/status.json?oauth={}&csrf={}", port, oauth_token, csrf_token);
    let res = client.get(&status_url).header("Origin", "https://open.spotify.com").send().await?.json::<SpotifyHandler>().await?;
    let action = if res.playing{true}else{false};
    let control_url = format!("https://local.spotify.com:{}/remote/puase.json?oauth={}&csrf={}&pause={}", port, oauth_token, csrf_token, action);
    let _ = client.get(&control_url).header("Origin", "https://open.spotify.com").send().await?;
    println!("Successfully togged playback!");
    Ok(())
}