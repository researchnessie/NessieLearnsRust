fn main(){
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon_with_id("assets/icon.ico","1");
        res.compile().unwrap();
    }
}