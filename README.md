# Tondi Dashboard

A modern, feature-rich dashboard for managing Tondi blockchain wallets and accounts. Built with Rust and WebAssembly for optimal performance and security.

## 🚀 Features

- **Multi-Platform Support**: Web application, Chrome extension, and native desktop app
- **Wallet Management**: Create, import, and manage Tondi wallets
- **Account Management**: Multiple account support with BIP32 derivation
- **Transaction Tools**: Send, receive, and monitor Tondi transactions
- **Real-time Updates**: Live blockchain data and wallet synchronization
- **Security**: Encrypted storage and secure key management
- **User-Friendly**: Modern UI built with egui framework

## 🛠️ Technology Stack

- **Backend**: Rust with async/await
- **Frontend**: WebAssembly (WASM) via egui
- **Blockchain**: Tondi chain integration
- **Storage**: Local encrypted storage
- **Networking**: gRPC client for node communication

## 📦 Installation

### Prerequisites

- Rust 1.80+ and Cargo
- Node.js 18+ (for web builds)
- Tondi node running locally or remotely

### Building from Source

```bash
# Clone the repository
git clone https://github.com/AvatoLabs/tondi-dashboard.git
cd tondi-dashboard

# Build the project
cargo build --release

# Build web application
cd app
trunk build

# Build Chrome extension
cd ../extensions/chrome
cargo build --target wasm32-unknown-unknown --release
```

### Running the Application

```bash
# Native desktop app
cargo run --bin tondi-dashboard

# Web application
cd app
trunk serve

# Chrome extension
# Load the extension from extensions/chrome/dist/ directory
```

## 🔧 Configuration

The dashboard connects to Tondi nodes via gRPC. Configure your node connection in the settings:

- **Network**: Mainnet or Testnet
- **Node URL**: Your Tondi node RPC endpoint
- **Connection Type**: Direct or proxy connection

## 📱 Usage

### Creating a Wallet

1. Launch the application
2. Click "Create New Wallet"
3. Set wallet name and security options
4. Generate or import mnemonic phrase
5. Create initial accounts

### Managing Accounts

- **Create Account**: Generate new BIP32 accounts
- **Import Account**: Import existing accounts via mnemonic
- **Account Switching**: Seamlessly switch between accounts
- **Address Management**: Generate new addresses as needed

### Sending Transactions

1. Select source account
2. Enter recipient address
3. Specify amount and priority fee
4. Review transaction details
5. Confirm and broadcast

## 🏗️ Architecture

```
tondi-dashboard/
├── app/                    # Web application
├── core/                   # Core framework
│   ├── modules/           # Feature modules
│   ├── runtime/           # Runtime services
│   └── utils/             # Utility functions
├── extensions/            # Browser extensions
│   └── chrome/           # Chrome extension
└── macros/               # Procedural macros
```

## 🔒 Security Features

- **Encrypted Storage**: All sensitive data is encrypted at rest
- **Secure Key Derivation**: BIP32/BIP39 compliant key generation
- **Memory Protection**: Secure memory handling and zeroization
- **Network Security**: Encrypted communication with nodes

## 🌐 Network Support

- **Mainnet**: Production Tondi network
- **Testnet**: Development and testing network
- **Custom Networks**: Support for custom node configurations

## 📊 Monitoring & Analytics

- **Real-time Balance**: Live account balance updates
- **Transaction History**: Complete transaction records
- **Network Status**: Node connection and sync status
- **Performance Metrics**: Transaction processing statistics

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Install development dependencies
cargo install trunk
cargo install wasm-bindgen-cli

# Run tests
cargo test

