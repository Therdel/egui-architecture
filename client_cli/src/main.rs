mod cli;
use clap::Parser;
use cli::{Cli, Command};
use client_core::{AppCore, PriceUpdate, ASSET_SYMBOLS};
use std::sync::mpsc;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::List => cmd_list(),
        Command::Fetch { symbol } => cmd_fetch(&symbol).await,
        Command::Watch { symbols } => cmd_watch(symbols).await,
    }
}

fn cmd_list() {
    println!("Known assets:");
    for asset in ASSET_SYMBOLS {
        println!("  {}", asset);
    }
}

async fn cmd_fetch(symbol: &str) {
    let symbol = symbol.to_uppercase();
    println!("Fetching price for {}...", symbol);
    match AppCore::fetch(&symbol).await {
        Ok(snap) => {
            let pct = snap.change_percent_24hr.parse::<f64>().unwrap_or(0.0);
            println!("  Symbol:     {}", snap.asset_symbol);
            println!("  Price:      ${:.2}", snap.price_usd.parse::<f64>().unwrap_or(0.0));
            println!("  24h High:   ${:.2}", snap.high_24hr.parse::<f64>().unwrap_or(0.0));
            println!("  24h Low:    ${:.2}", snap.low_24hr.parse::<f64>().unwrap_or(0.0));
            println!("  24h Change: {:+.2}%", pct);
            println!("  24h Volume: ${:.0}", snap.volume_usd_24hr.parse::<f64>().unwrap_or(0.0));
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

async fn cmd_watch(symbols: Vec<String>) {
    let symbols_lc: Vec<String> = symbols.iter().map(|s| s.to_lowercase()).collect();
    let symbols_ref: Vec<&str> = symbols_lc.iter().map(String::as_str).collect();
    println!("Watching {} (Ctrl-C to stop)...", symbols.join(", "));

    let (tx, rx) = mpsc::channel::<PriceUpdate>();
    AppCore::watch(&symbols_ref, tx);

    loop {
        while let Ok(update) = rx.try_recv() {
            println!("  {} = ${:.2}", update.asset_symbol, update.price_usd.parse::<f64>().unwrap_or(0.0));
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}

