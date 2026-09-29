//a way to store reminders and their types (prob JSON)

//time and date-time to trigger reminders

//types of reminder aka recurring/one time/deadline

//a system to keep loopable reminders (daily, monthly, weekly etc...)

//gui for input and display with categories or priority system

//repeated reminders for missed notifications aka auto-snooze I guess

use eframe::egui;
pub fn remind(){
    let options = eframe::NativeOptions{
        viewport: egui::ViewportBuilder::default().with_inner_size([400.0,300.0]),
        ..Default::default()
    };
    eframe::run_native("TodoList", options, Box::new(|_cc| Ok(Box::<TodoList>::default())));
}

struct TodoList{
    balance: f32,
}

impl Default for TodoList {
    fn default() -> Self {
        Self {balance :100.00}
    }
}

impl eframe::App for TodoList {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
            ui.heading("My greeting app");
            ui.label("Hi there!");
    }
}