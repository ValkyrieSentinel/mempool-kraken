use crate::mempool::PendingTx;
use ethers::types::{H160, H256, U256};
use std::collections::HashMap;
use tracing::{info, warn};

const SELECTOR_SWAP_EXACT_TOKENS_FOR_TOKENS: [u8; 4] = [0x38, 0xed, 0x17, 0x39];
const SELECTOR_SWAP_EXACT_ETH_FOR_TOKENS: [u8; 4] = [0x7f, 0xf3, 0x6a, 0xb5];
const SELECTOR_EXACT_INPUT_SINGLE: [u8; 4] = [0x41, 0x4b, 0xf3, 0x89];


const HIGH_PRIORITY_FEE_THRESHOLD: u64 = 1_000_000_000;

#[derive(Debug, Clone, PartialEq)]
pub enum SuspicionLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub tx_hash: H256,
    pub from: H160,
    pub suspicion: SuspicionLevel,
    pub reason: String,
    pub is_known_swap: bool,
}

pub struct Detector {
    sender_tx_count: HashMap<H160, u32>,
}

impl Detector {
    pub fn new() -> Self {
        Self {
            sender_tx_count: HashMap::new(),
        }
    }

    pub fn analyze(&mut self, tx: &PendingTx) -> AnalysisResult {
        let mut score = 0;
        let mut reasons = Vec::new();
        let mut is_known_swap = false;

        if tx.input.len() >= 4 {
            let selector = &tx.input[..4];
            if selector == SELECTOR_SWAP_EXACT_TOKENS_FOR_TOKENS
                || selector == SELECTOR_SWAP_EXACT_ETH_FOR_TOKENS
                || selector == SELECTOR_EXACT_INPUT_SINGLE
            {
                is_known_swap = true;
                score += 2;
                reasons.push("Contains known DEX swap selector".to_string());
            }
        }

        if let Some(priority_fee) = tx.max_priority_fee_per_gas {
            if priority_fee >= U256::from(HIGH_PRIORITY_FEE_THRESHOLD) {
                score += 3;
                reasons.push(format!("High priority fee (bribe): {} Wei", priority_fee));
            }
        } else if let Some(gas_price) = tx.gas_price {
            if gas_price >= U256::from(HIGH_PRIORITY_FEE_THRESHOLD * 2) {
                score += 2;
                reasons.push(format!("High legacy gas price: {} Wei", gas_price));
            }
        }

        let count = self.sender_tx_count.entry(tx.from).or_insert(0);
        *count += 1;
        if *count > 5 {
            score += 1;
            reasons.push(format!("Sender is highly active ({} recent txs)", *count));
        }

        let suspicion = match score {
            0..=2 => SuspicionLevel::Low,
            3..=4 => SuspicionLevel::Medium,
            5..=6 => SuspicionLevel::High,
            _ => SuspicionLevel::Critical,
        };

        if suspicion == SuspicionLevel::High || suspicion == SuspicionLevel::Critical {
            warn!(
                "🚨 MEV SUSPICION DETECTED! Hash: {:?} | From: {:?} | Level: {:?} | Reasons: {:?}",
                tx.hash, tx.from, suspicion, reasons
            );
        }

        AnalysisResult {
            tx_hash: tx.hash,
            from: tx.from,
            suspicion,
            reason: reasons.join("; "),
            is_known_swap,
        }
    }
    #[allow(dead_code)]

    pub fn reset_tracking(&mut self) {
        self.sender_tx_count.clear();
        info!("Detector tracking map reset");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_low_suspicion() {
        let mut detector = Detector::new();
        let tx = PendingTx {
            hash: H256::zero(),
            from: H160::zero(),
            to: None,
            value: U256::zero(),
            input: vec![0x00, 0x00, 0x00, 0x00],
            gas: U256::from(21000),
            gas_price: Some(U256::from(20_000_000_000u64)),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: Some(U256::from(1_000_000u64)),
            nonce: U256::zero(),
            is_dex_interaction: false,
        };

        let result = detector.analyze(&tx);
        assert_eq!(result.suspicion, SuspicionLevel::Low);
        assert!(!result.is_known_swap);
    }

    #[test]
    fn test_detector_high_suspicion() {
        let mut detector = Detector::new();
        let mut input = Vec::new();
        input.extend_from_slice(&SELECTOR_SWAP_EXACT_TOKENS_FOR_TOKENS);
        input.extend_from_slice(&[0; 32]);

        let tx = PendingTx {
            hash: H256::zero(),
            from: H160::zero(),
            to: None,
            value: U256::zero(),
            input,
            gas: U256::from(200000),
            gas_price: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: Some(U256::from(5_000_000_000u64)),
            nonce: U256::zero(),
            is_dex_interaction: true,
        };

        let result = detector.analyze(&tx);
        assert_eq!(result.suspicion, SuspicionLevel::High);
        assert!(result.is_known_swap);
        assert!(result.reason.contains("High priority fee"));
    }
}