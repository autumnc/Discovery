mod app;
mod config;
mod constants;
mod http;
mod model;
mod parser;
mod theme;
mod ui;
mod utils;

use clap::Parser;

#[derive(Parser)]
#[command(name = "hipda-tui", about = "TUI forum client for HiPDA (4d4y.com)")]
struct Args {
    #[arg(short, long, default_value_t = String::new())]
    username: String,

    #[arg(short, long, default_value_t = String::new())]
    password: String,

    #[arg(long, default_value_t = false)]
    re_login: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut cfg = config::Config::load()?;
    if !args.username.is_empty() {
        cfg.username = args.username;
    }
    if !args.password.is_empty() {
        cfg.password = args.password;
    }

    if args.re_login {
        cfg.clear_auth();
    }

    app::run(cfg).await
}
