mod mempool;
mod detector;
mod guardian;
mod alerts;

use dotenvy::dotenv;
use std::env;
use tracing::info;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    info!("🐙 Mempool Kraken starting up...");

    let tg_token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set in .env");
    let tg_chat_id = env::var("TELEGRAM_CHAT_ID").expect("TELEGRAM_CHAT_ID must be set in .env");
    
    let mempool_config = mempool::MempoolConfig::default();
    info!("Monitoring {} DEX routers...", mempool_config.target_dex_contracts.len());

    let alerter = alerts::TelegramAlerter::new(alerts::AlerterConfig {
        bot_token: tg_token,
        chat_id: tg_chat_id,
    });
    
    alerter.send_startup_message().await;

    let mut detector = detector::Detector::new();
    let guardian = guardian::Guardian::new();

    info!("Connecting to mempool stream...");
    let mut rx = match mempool::start_mempool_listener(mempool_config).await {
        Ok(receiver) => receiver,
        Err(e) => {
            tracing::error!("Failed to start mempool listener: {}", e);
            std::process::exit(1);
        }
    };

    info!("Kraken is hunting. Waiting for transactions...");
    let mut tx_count = 0;

    while let Some(tx) = rx.recv().await {
        tx_count += 1;
        if tx_count % 100 == 0 {
            info!("Processed {} transactions...", tx_count);
        }

        let analysis = detector.analyze(&tx);
        
        if analysis.suspicion == detector::SuspicionLevel::High 
            || analysis.suspicion == detector::SuspicionLevel::Critical 
        {
            info!("🚨 High suspicion detected! Sending alert...");
            alerter.send_detector_alert(&analysis).await;
        }

        if let Some(alert) = guardian.analyze(&tx) {
            info!("️ Guardian alert triggered! Sending alert...");
            alerter.send_guardian_alert(&alert).await;
        }
    }

    tracing::error!("Mempool stream closed unexpectedly. Shutting down.");
}