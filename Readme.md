# Wallet

A Rust-based password management tool for securely storing and managing your credentials.

## Features

- Secure password storage with encryption
- Master password protection
- Password generation
- Credential management and organization
- Cross-platform compatibility

## Installation

```bash
git clone <repository-url>
cd wallet
cargo build --release
```

## Usage

```bash
# Password will be prompted securely (hidden input)
wallet add my-key my-value
wallet list
wallet show my-key
wallet generate -l 32 -s

# Or via environment variable for scripting:
export WALLET_PASSWORD=my-secret
wallet add api-key ghp_xxxx
```

## Storage

Data is encrypted with AES-256-GCM and stored at:
```
~/.local/share/wallet/data.wallet
```
The master password is derived into a 256-bit key via PBKDF2 (100,000 iterations).

## Testing

```bash
cargo test
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests
5. Submit a pull request

## License

This project is licensed under the MIT License.
