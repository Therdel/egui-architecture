use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Binance market data CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Print the supported asset symbols
    List,
    /// Fetch the latest price snapshot for an asset (e.g. BTCUSDT)
    Fetch { symbol: String },
    /// Watch live price updates for one or more assets (Ctrl-C to stop)
    Watch {
        /// One or more symbols to watch, e.g. BTCUSDT ETHUSDT
        #[arg(required = true)]
        symbols: Vec<String>,
    },
}
