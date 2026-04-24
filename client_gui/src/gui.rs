use eframe::egui;
use client_core::{AppCore, AssetSnapshot, PriceUpdate, ASSET_SYMBOLS};
use std::sync::mpsc::{self, Receiver};
use std::collections::HashMap;

pub struct Gui {
    asset: Option<Result<AssetSnapshot, String>>,
    asset_rx: Option<Receiver<Result<AssetSnapshot, String>>>,
    prices: HashMap<String, String>,
    price_rx: Receiver<PriceUpdate>,
    #[cfg(not(target_arch = "wasm32"))]
    rt: tokio::runtime::Runtime,
}

impl Gui {
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

        let (price_tx, price_rx) = mpsc::channel();
        let assets_ws: Vec<String> = ASSET_SYMBOLS.iter().map(|s| s.to_lowercase()).collect();
        let assets_ws_ref: Vec<&str> = assets_ws.iter().map(String::as_str).collect();

        #[cfg(not(target_arch = "wasm32"))]
        let _guard = rt.enter();
        AppCore::watch(&assets_ws_ref, price_tx);
        #[cfg(not(target_arch = "wasm32"))]
        drop(_guard);

        Self {
            asset: None,
            asset_rx: None,
            prices: HashMap::new(),
            price_rx,
            #[cfg(not(target_arch = "wasm32"))]
            rt,
        }
    }

    fn fetch(&mut self, asset: &'static str) {
        let (tx, rx) = mpsc::channel();
        self.asset_rx = Some(rx);
        self.asset = None;

        #[cfg(not(target_arch = "wasm32"))]
        self.rt.spawn(async move {
            let result = AppCore::fetch(asset).await;
            let _ = tx.send(result);
        });

        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            let result = AppCore::fetch(asset).await;
            let _ = tx.send(result);
        });
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll asset price result
        if let Some(rx) = &self.asset_rx {
            if let Ok(result) = rx.try_recv() {
                self.asset = Some(result);
                self.asset_rx = None;
            }
        }

        // Drain live price updates
        while let Ok(update) = self.price_rx.try_recv() {
            self.prices.insert(update.asset_symbol, update.price_usd);
        }

        // Keep polling while loading or streaming
        if self.asset_rx.is_some() || !self.prices.is_empty() {
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Binance Live");
            ui.separator();

            // ── Snapshot panel ────────────────────────────────────────────
            ui.label(egui::RichText::new("Asset Snapshot").strong());
            ui.horizontal(|ui| {
                for &asset in ASSET_SYMBOLS {
                    if ui.button(asset).clicked() {
                        self.fetch(asset);
                    }
                }
            });

            match &self.asset {
                None if self.asset_rx.is_some() => { ui.spinner(); }
                Some(Ok(snap)) => {
                    egui::Grid::new("snapshot_grid").num_columns(2).striped(true).show(ui, |ui| {
                        ui.label("Symbol"); ui.label(&snap.asset_symbol); ui.end_row();
                        ui.label("Price (USD)"); ui.label(format!("${:.2}", snap.price_usd.parse::<f64>().unwrap_or(0.0))); ui.end_row();
                        ui.label("24h High"); ui.label(format!("${:.2}", snap.high_24hr.parse::<f64>().unwrap_or(0.0))); ui.end_row();
                        ui.label("24h Low"); ui.label(format!("${:.2}", snap.low_24hr.parse::<f64>().unwrap_or(0.0))); ui.end_row();
                        ui.label("24h Change");
                        let pct = snap.change_percent_24hr.parse::<f64>().unwrap_or(0.0);
                        let color = if pct >= 0.0 { egui::Color32::GREEN } else { egui::Color32::RED };
                        ui.colored_label(color, format!("{:+.2}%", pct));
                        ui.end_row();
                        ui.label("24h Volume"); ui.label(format!("${:.0}", snap.volume_usd_24hr.parse::<f64>().unwrap_or(0.0))); ui.end_row();
                    });
                }
                Some(Err(e)) => { ui.colored_label(egui::Color32::RED, e); }
                None => { ui.label("Select an asset above."); }
            }

            ui.add_space(12.0);
            ui.separator();

            // ── Live prices ───────────────────────────────────────────────
            ui.label(egui::RichText::new("Live Prices (WebSocket)").strong());
            if self.prices.is_empty() {
                ui.spinner();
            } else {
                egui::Grid::new("prices_grid").num_columns(2).striped(true).show(ui, |ui| {
                    let mut sorted: Vec<_> = self.prices.iter().collect();
                    sorted.sort_by_key(|(k, _)| k.as_str());
                    for (asset, price) in sorted {
                        ui.label(asset);
                        ui.label(format!("${:.2}", price.parse::<f64>().unwrap_or(0.0)));
                        ui.end_row();
                    }
                });
            }
        });
    }
}

