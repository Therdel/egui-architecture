mod api;
pub use api::{AssetSnapshot, PriceUpdate, ASSET_SYMBOLS};
use api::{StreamMessage};

use std::sync::mpsc::Sender;

pub struct AppCore;

impl AppCore {
    pub fn new() -> Self {
        Self
    }

    /// Fetches a 24hr ticker snapshot via REST. `symbol` is e.g. `"BTCUSDT"`.
    pub async fn fetch(symbol: &str) -> Result<AssetSnapshot, String> {
        let url = format!("https://api.binance.com/api/v3/ticker/24hr?symbol={}", symbol);
        let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
        resp.json().await.map_err(|e| e.to_string())
    }

    /// Connects to the Binance combined stream and sends `PriceUpdate`s into `tx`.
    /// `symbols` should be lowercase Binance pairs, e.g. `&["btcusdt", "ethusdt"]`.
    pub fn watch(symbols: &[&str], tx: Sender<PriceUpdate>) {
        let streams = symbols
            .iter()
            .map(|s| format!("{}@miniTicker", s))
            .collect::<Vec<_>>()
            .join("/");
        let url = format!("wss://stream.binance.com/stream?streams={}", streams);

        let (ws_tx, ws_rx) = ewebsock::connect(url, ewebsock::Options::default())
            .expect("Failed to connect to Binance WebSocket");

        #[cfg(not(target_arch = "wasm32"))]
        tokio::spawn(async move {
            let _keep_alive = ws_tx;
            pump_ws(ws_rx, tx, || tokio::time::sleep(std::time::Duration::from_millis(50))).await;
        });

        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            let _keep_alive = ws_tx;
            pump_ws(ws_rx, tx, || gloo_timers::future::TimeoutFuture::new(50)).await;
        });
    }
}

async fn pump_ws<F, Fut>(ws_rx: ewebsock::WsReceiver, tx: Sender<PriceUpdate>, sleep: F)
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    loop {
        while let Some(event) = ws_rx.try_recv() {
            match event {
                ewebsock::WsEvent::Message(ewebsock::WsMessage::Text(text)) => {
                    if let Ok(msg) = serde_json::from_str::<StreamMessage>(&text) {
                        let update = PriceUpdate { asset_symbol: msg.data.symbol, price_usd: msg.data.close };
                        if tx.send(update).is_err() {
                            return;
                        }
                    }
                }
                ewebsock::WsEvent::Closed | ewebsock::WsEvent::Error(_) => return,
                _ => {}
            }
        }
        sleep().await;
    }
}
