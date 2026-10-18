use crate::mempool::PendingTx;
use ethers::types::{H160, H256};
use std::collections::HashMap;
use tracing::info;

const SELECTOR_RENOUNCE_OWNERSHIP: [u8; 4] = [0x71, 0x50, 0x18, 0xa6];
const SELECTOR_TRANSFER_OWNERSHIP: [u8; 4] = [0xf2, 0xf3, 0x2a, 0x09];
const SELECTOR_UPGRADE_TO: [u8; 4] = [0x36, 0x59, 0xc8, 0xfe];
const SELECTOR_UPGRADE_TO_AND_CALL: [u8; 4] = [0x4f, 0x1e, 0xf2, 0x11];
const SELECTOR_PAUSE: [u8; 4] = [0x84, 0x56, 0xcb, 0x59];

#[derive(Debug, Clone, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone)]
pub struct GuardianAlert {
    pub tx_hash: H256,
    pub target_contract: H160,
    pub severity: AlertSeverity,
    pub threat_type: String,
    pub description: String,
}

pub struct Guardian {
    watched_contracts: HashMap<H160, String>,
}

impl Guardian {
    pub fn new() -> Self {
        Self {
            watched_contracts: HashMap::new(),
        }
    }
    #[allow(dead_code)]

    pub fn watch_contract(&mut self, address: H160, name: String) {
    self.watched_contracts.insert(address, name.clone());
    info!("Guardian is now watching: {} ({})", name, address);
}

    pub fn analyze(&self, tx: &PendingTx) -> Option<GuardianAlert> {
        let target = match tx.to {
            Some(addr) => addr,
            None => return None,
        };

        if !self.watched_contracts.contains_key(&target) {
            return None;
        }

        if tx.input.len() < 4 {
            return None;
        }

        let selector = &tx.input[..4];
        let token_name = self.watched_contracts.get(&target).unwrap().clone();

        if selector == SELECTOR_RENOUNCE_OWNERSHIP {
            return Some(GuardianAlert {
                tx_hash: tx.hash,
                target_contract: target,
                severity: AlertSeverity::Warning,
                threat_type: "Ownership Renounced".to_string(),
                description: format!("Owner of {} is renouncing ownership. Verify liquidity lock!", token_name),
            });
        }

        if selector == SELECTOR_TRANSFER_OWNERSHIP {
            return Some(GuardianAlert {
                tx_hash: tx.hash,
                target_contract: target,
                severity: AlertSeverity::Info,
                threat_type: "Ownership Transfer".to_string(),
                description: format!("Ownership of {} is being transferred.", token_name),
            });
        }

        if selector == SELECTOR_UPGRADE_TO || selector == SELECTOR_UPGRADE_TO_AND_CALL {
            return Some(GuardianAlert {
                tx_hash: tx.hash,
                target_contract: target,
                severity: AlertSeverity::Critical,
                threat_type: "Proxy Upgrade".to_string(),
                description: format!("CRITICAL: {} is being upgraded! The contract logic can be completely changed.", token_name),
            });
        }

        if selector == SELECTOR_PAUSE {
            return Some(GuardianAlert {
                tx_hash: tx.hash,
                target_contract: target,
                severity: AlertSeverity::Warning,
                threat_type: "Contract Paused".to_string(),
                description: format!("Trading on {} has been paused by the owner.", token_name),
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guardian_detects_proxy_upgrade() {
        let mut guardian = Guardian::new();
        let target_addr = "0x1234567890123456789012345678901234567890".parse::<H160>().unwrap();
        guardian.watch_contract(target_addr, "SuspiciousToken".to_string());

        let mut input = Vec::new();
        input.extend_from_slice(&SELECTOR_UPGRADE_TO);
        input.extend_from_slice(&[0; 32]);

        let tx = PendingTx {
            hash: H256::zero(),
            from: H160::zero(),
            to: Some(target_addr),
            value: ethers::types::U256::zero(),
            input,
            gas: ethers::types::U256::from(100000),
            gas_price: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            nonce: ethers::types::U256::zero(),
            is_dex_interaction: false,
        };

        let alert = guardian.analyze(&tx);
        assert!(alert.is_some());
        let alert = alert.unwrap();
        assert_eq!(alert.severity, AlertSeverity::Critical);
        assert_eq!(alert.threat_type, "Proxy Upgrade");
    }

    #[test]
    fn test_guardian_ignores_unwatched_contract() {
        let guardian = Guardian::new();
        let target_addr = "0x1234567890123456789012345678901234567890".parse::<H160>().unwrap();

        let tx = PendingTx {
            hash: H256::zero(),
            from: H160::zero(),
            to: Some(target_addr),
            value: ethers::types::U256::zero(),
            input: SELECTOR_RENOUNCE_OWNERSHIP.to_vec(),
            gas: ethers::types::U256::from(100000),
            gas_price: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            nonce: ethers::types::U256::zero(),
            is_dex_interaction: false,
        };

        let alert = guardian.analyze(&tx);
        assert!(alert.is_none());
    }
}