use serde::Deserialize;

/// Canonical list of tracked assets (uppercase Binance pairs).
pub const ASSET_SYMBOLS: &[&str] = &["BTCUSDT", "ETHUSDT", "SOLUSDT", "BNBUSDT", "XRPUSDT"];

// ── REST ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Deserialize)]
pub struct AssetSnapshot {
    #[serde(rename = "symbol")]
    pub asset_symbol: String,
    #[serde(rename = "lastPrice")]
    pub price_usd: String,
    #[serde(rename = "priceChangePercent")]
    pub change_percent_24hr: String,
    #[serde(rename = "highPrice")]
    pub high_24hr: String,
    #[serde(rename = "lowPrice")]
    pub low_24hr: String,
    #[serde(rename = "quoteVolume")]
    pub volume_usd_24hr: String,
}

// ── WebSocket ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct PriceUpdate {
    pub asset_symbol: String,
    pub price_usd: String,
}

// ── Internal Binance wire types ───────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct StreamMessage {
    pub(crate) data: MiniTicker,
}

#[derive(Deserialize)]
pub(crate) struct MiniTicker {
    #[serde(rename = "s")]
    pub(crate) symbol: String,
    #[serde(rename = "c")]
    pub(crate) close: String,
}
