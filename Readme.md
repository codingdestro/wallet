# Wallet

A Rust-based password management tool for securely storing and managing your credentials.

## Features

- 🔒 Secure password storage with AES-256-GCM encryption
- 🔑 Master password protection with PBKDF2 key derivation
- 🎲 Password generation (alphanumeric or with special characters)
- 📋 Clipboard integration for easy copying
- 🖥️ **Terminal UI** with interactive password management
- ⌨️ **CLI mode** with subcommands for scripting
- 📁 Credential management and organization

## Installation

```bash
git clone <repository-url>
cd wallet
cargo build --release
```

## Usage

### CLI Mode (default)

```bash
# Password will be prompted securely (hidden input)
wallet add my-key my-value
wallet list
wallet show my-key
wallet remove my-key
wallet copy my-key
wallet generate -l 32 -s

# Or via environment variable for scripting:
export WALLET_PASSWORD=my-secret
wallet add api-key ghp_xxxx
```

### TUI Mode (interactive)

Run without arguments or with `--tui` / `-t`:

```bash
wallet           # auto-launches TUI
wallet --tui     # explicit TUI mode
```

| Key | Action |
|-----|--------|
| `↑/↓` / `j/k` | Navigate entries |
| `Enter` | View entry detail |
| `a` | Add new entry |
| `d` | Delete selected entry |
| `c` | Copy value to clipboard |
| `Tab` / `h` | Show/hide value |
| `q` / `Esc` | Quit |

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
