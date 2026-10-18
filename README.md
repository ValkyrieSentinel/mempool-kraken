# Mempool Kraken

Real-time MEV and sandwich-attack detector for the Ethereum mempool.

## Features

- Real-time Mempool Streaming via WebSocket
- Heuristic MEV Detection (gas analysis, DEX selectors)
- Guardian Module for contract monitoring
- Instant Telegram Alerts

## Architecture

1. Mempool Listener (src/mempool.rs)
2. Detector (src/detector.rs)
3. Guardian (src/guardian.rs)
4. Alerter (src/alerts.rs)

## Getting Started

### Prerequisites
- Rust toolchain (cargo)
- Alchemy WebSocket URL
- Telegram Bot Token and Chat ID

### Installation and Run

1. Clone:
   git clone https://github.com/ValkyrieSentinel/mempool-kraken.git
   cd mempool-kraken

2. Create .env:
   ETH_WS_URL=wss://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY
   TELEGRAM_BOT_TOKEN=your_token
   TELEGRAM_CHAT_ID=your_id

3. Run:
   cargo run --release

## Disclaimer

Educational tool for security research. Not a competitive MEV searcher.

## License

MIT License. Part of the ValkyrieSentinel ecosystem.
