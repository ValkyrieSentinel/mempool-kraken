use ethers::{
    prelude::*,
    providers::{Provider, Ws},
    types::{H160, H256, U256},
};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

#[derive(Clone, Debug)]
pub struct MempoolConfig {
    pub ws_url: String,
    pub target_dex_contracts: Vec<H160>,
}

impl Default for MempoolConfig {
    fn default() -> Self {
        Self {
            ws_url: std::env::var("ETH_WS_URL")
                .unwrap_or_else(|_| "wss://eth-mainnet.g.alchemy.com/v2/demo".to_string()),
            target_dex_contracts: vec![
                "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D".parse().unwrap(),
                "0xE592427A0AEce92De3Edee1F18E0157C05861564".parse().unwrap(),
                "0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45".parse().unwrap(),
            ],
        }
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]

pub struct PendingTx {
    pub hash: H256,
    pub from: H160,
    pub to: Option<H160>,
    pub value: U256,
    pub input: Vec<u8>,
    pub gas: U256,
    pub gas_price: Option<U256>,
    pub max_fee_per_gas: Option<U256>,
    pub max_priority_fee_per_gas: Option<U256>,
    pub nonce: U256,
    pub is_dex_interaction: bool,
}

pub struct MempoolListener {
    config: MempoolConfig,
    provider: Arc<Provider<Ws>>,
    tx_sender: mpsc::Sender<PendingTx>,
}

impl MempoolListener {
    
    pub fn new(config: MempoolConfig, provider: Arc<Provider<Ws>>, tx_sender: mpsc::Sender<PendingTx>) -> Self {
        Self {
            config,
            provider,
            tx_sender,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Mempool listener started. Monitoring {} DEX contracts...", self.config.target_dex_contracts.len());

        let mut stream = self.provider.subscribe_pending_txs().await?;
        info!("Subscribed to pending transactions stream");

        while let Some(tx_hash) = stream.next().await {
            if let Err(e) = self.process_pending_tx(tx_hash).await {
                warn!("Failed to process tx {}: {}", tx_hash, e);
            }
        }

        Ok(())
    }

    async fn process_pending_tx(
        &self,
        hash: H256,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tx: Option<Transaction> = self.provider
            .get_transaction(hash)
            .await?;

        let tx = match tx {
            Some(t) => t,
            None => return Ok(()),
        };

        let is_dex = tx
            .to
            .map(|to| self.config.target_dex_contracts.contains(&to))
            .unwrap_or(false);

        let pending = PendingTx {
            hash: tx.hash,
            from: tx.from,
            to: tx.to,
            value: tx.value,
            input: tx.input.to_vec(),
            gas: tx.gas,
            gas_price: tx.gas_price,
            max_fee_per_gas: tx.max_fee_per_gas,
            max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
            nonce: tx.nonce,
            is_dex_interaction: is_dex,
        };

        if is_dex {
            info!(
                "DEX tx detected: from={} to={:?} gas_price={:?}",
                pending.from, pending.to, pending.gas_price
            );
        }

        let _ = self.tx_sender.try_send(pending);

        Ok(())
    }
}


pub async fn start_mempool_listener(
    config: MempoolConfig,
) -> Result<mpsc::Receiver<PendingTx>, Box<dyn std::error::Error>> {
    let ws = Ws::connect(&config.ws_url).await?;
    let provider = Arc::new(Provider::new(ws));

    let (tx, rx) = mpsc::channel(1024);
    let listener = MempoolListener::new(config, provider, tx);

    tokio::spawn(async move {
        if let Err(e) = listener.run().await {
            error!("Mempool listener crashed: {}", e);
        }
    });

    Ok(rx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MempoolConfig::default();
        assert_eq!(config.target_dex_contracts.len(), 3);
    }
}