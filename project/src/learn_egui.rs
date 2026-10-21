use eframe::egui;

pub struct MyApp{
    name:String,
    clicks:u32
}
impl MyApp{
    fn default() -> Self{
        Self{
            name:"".to_string(),clicks:0
        }
    }
}
impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        let list1 = ["hello", "world","welcome","to","rust"];
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Lesson 2");

            ui.vertical(|ui| {
                ui.label("Your name:");
                ui.text_edit_singleline(&mut self.name);
                ui.hyperlink("https://github.com/researchnessie");
            });
            ui.vertical(|ui| {
                for l in list1.iter(){
                    ui.label(*l);
                }
            });

            if ui.button("Click me").clicked() {
                self.clicks += 1;
            }

            ui.label(format!("Hello {}! Clicks: {}", self.name, self.clicks));
        });
    }
}

pub fn learn()->eframe::Result{
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Mi first app!", options, Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}