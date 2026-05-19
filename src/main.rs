mod app;
mod crdt;
mod network;

use anyhow::Result;
use app::TexasApp;
use tokio::runtime::Builder;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let runtime = Builder::new_multi_thread().enable_all().build()?;

    eframe::run_native(
        "TeXas",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("TeXas")
                .with_inner_size([1360.0, 860.0])
                .with_min_inner_size([960.0, 640.0]),
            ..Default::default()
        },
        Box::new(move |cc| match TexasApp::new(&cc.egui_ctx, runtime) {
            Ok(app) => Ok(Box::new(app)),
            Err(error) => Err(Box::new(std::io::Error::other(error.to_string()))),
        }),
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;

    Ok(())
}
