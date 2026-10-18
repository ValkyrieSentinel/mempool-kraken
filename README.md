# Mempool Kraken

MEV bot & sandwich-attack hunter with an on-chain smart-contract guardian. Real-time mempool surveillance written in Rust.

## What it hunts
- Sandwich attacks: detects buy/sell pairs wrapping a victim transaction inside the same block.
- MEV bots: flags accounts with abnormal priority-fee bribes and repeated backrunning patterns.
- Contract threats: watches guarded contracts for ownership transfers, proxy upgrades, and honeypot-style sell blocks.

## Architecture
- mempool: pending transaction stream (WebSocket).
- detector: sandwich & MEV heuristics (gas-bribe analysis, token flow symmetry).
- guardian: smart-contract watchdog (events, ownership, upgrades).
- alerts: Telegram notifications.

## Status
Under active development. Built file-by-file in public.

## License
MIT. Part of the ValkyrieSentinel ecosystem.
