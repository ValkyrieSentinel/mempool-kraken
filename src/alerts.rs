use crate::detector::{AnalysisResult, SuspicionLevel};
use crate::guardian::{AlertSeverity, GuardianAlert};
use reqwest::Client;
use tracing::{error, info};

pub struct AlerterConfig {
    pub bot_token: String,
    pub chat_id: String,
}

pub struct TelegramAlerter {
    client: Client,
    base_url: String,
    chat_id: String,
}

impl TelegramAlerter {
    pub fn new(config: AlerterConfig) -> Self {
        let base_url = format!("https://api.telegram.org/bot{}/sendMessage", config.bot_token);
        Self {
            client: Client::new(),
            base_url,
            chat_id: config.chat_id,
        }
    }

    async fn send_message(&self, text: String) -> Result<(), reqwest::Error> {
        let params = [
            ("chat_id", self.chat_id.as_str()),
            ("text", text.as_str()),
            ("parse_mode", "HTML"),
        ];

        let res = self.client.post(&self.base_url).form(&params).send().await?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            error!("Telegram API error: {} - {}", status, body);
        } else {
            info!("Alert sent successfully to Telegram");
        }

        Ok(())
    }

    pub async fn send_detector_alert(&self, result: &AnalysisResult) {
        let etherscan_url = format!("https://etherscan.io/tx/{:?}", result.tx_hash);
        let from_url = format!("https://etherscan.io/address/{:?}", result.from);
        
        let emoji = match result.suspicion {
            SuspicionLevel::Low => "",
            SuspicionLevel::Medium => "🟡",
            SuspicionLevel::High => "🟠",
            SuspicionLevel::Critical => "",
        };

        let message = format!(
            "{} <b>MEV / Sandwich Suspicion</b>\n\n\
             <b>Level:</b> {:?}\n\
             <b>From:</b> <a href=\"{}\">{:?}</a>\n\
             <b>Tx:</b> <a href=\"{}\">View on Etherscan</a>\n\
             <b>Reason:</b> {}\n\
             <b>Known Swap:</b> {}",
            emoji,
            result.suspicion,
            from_url,
            result.from,
            etherscan_url,
            if result.reason.is_empty() { "None".to_string() } else { result.reason.clone() },
            result.is_known_swap
        );

        if let Err(e) = self.send_message(message).await {
            error!("Failed to send detector alert: {}", e);
        }
    }

    pub async fn send_guardian_alert(&self, alert: &GuardianAlert) {
        let etherscan_tx = format!("https://etherscan.io/tx/{:?}", alert.tx_hash);
        let etherscan_contract = format!("https://etherscan.io/address/{:?}", alert.target_contract);

        let emoji = match alert.severity {
            AlertSeverity::Info => "️",
            AlertSeverity::Warning => "️",
            AlertSeverity::Critical => "🚨",
        };

        let message = format!(
            "{} <b>Guardian Alert: {}</b>\n\n\
             <b>Severity:</b> {:?}\n\
             <b>Target:</b> <a href=\"{}\">{:?}</a>\n\
             <b>Tx:</b> <a href=\"{}\">View on Etherscan</a>\n\
             <b>Description:</b> {}",
            emoji,
            alert.threat_type,
            alert.severity,
            etherscan_contract,
            alert.target_contract,
            etherscan_tx,
            alert.description
        );

        if let Err(e) = self.send_message(message).await {
            error!("Failed to send guardian alert: {}", e);
        }
    }

    pub async fn send_startup_message(&self) {
        let message = " <b>Mempool Kraken is online!</b>\nListening to the mempool and guarding contracts...".to_string();
        let _ = self.send_message(message).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alerter_creation() {
        let config = AlerterConfig {
            bot_token: "test_token".to_string(),
            chat_id: "test_chat".to_string(),
        };
        let alerter = TelegramAlerter::new(config);
        assert_eq!(alerter.chat_id, "test_chat");
        assert!(alerter.base_url.contains("test_token"));
    }
}