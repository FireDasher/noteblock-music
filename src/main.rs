mod app;
mod project;

fn main() {
    let iconbytes = include_bytes!("noteblock.bin");
    let icon = eframe::egui::IconData {
        width: 16,
        height: 16,
        rgba: iconbytes.to_vec()
    };
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder {
            icon: Some(std::sync::Arc::new(icon)),
            ..Default::default()
        },
        ..Default::default()
    };
	eframe::run_native("Note Block Music Thing", options, Box::new(|cc| Ok(Box::new(app::App::new(cc))))).expect("Failed to run App!");
}