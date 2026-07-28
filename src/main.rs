mod app;
mod math;
mod plot;
mod store;
mod ui;

use anyhow::{anyhow, Result};
use eframe::egui;

fn main() -> Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Rust Native Graphing Calculator")
            .with_inner_size([1080.0, 720.0])
            .with_min_inner_size([820.0, 560.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Rust Native Graphing Calculator",
        native_options,
        Box::new(|cc| {
            ui::install_fonts(&cc.egui_ctx);
            ui::install_visuals(&cc.egui_ctx, true);
            Ok(Box::new(app::App::new(cc)))
        }),
    )
    .map_err(|err| anyhow!(err.to_string()))?;

    Ok(())
}
