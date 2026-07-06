# wallet — Encrypted Secret Manager

A CLI tool for managing passwords, API keys, and secrets in an **AES-256-GCM** encrypted file. Secrets are decrypted in memory only — plaintext never touches disk.

## Install

```bash
# From npm
npm install -g @codingdestro/wallet

# Or run directly
npx @codingdestro/wallet <command>
```

## Quick Start

```bash
# Create a new encrypted wallet (you'll be prompted for a password)
wallet init

# Add a secret
wallet add my-api-key

# List stored keys
wallet list

# Copy a secret to clipboard
wallet copy my-api-key

# Delete a secret
wallet delete my-api-key
```

## Commands

| Command               | Alias | Description                        |
| --------------------- | ----- | ---------------------------------- |
| `wallet init`         |       | Create a new encrypted wallet file |
| `wallet add <key>`    | `a`   | Add or update a secret             |
| `wallet list`         | `l`   | List all stored key names          |
| `wallet copy <key>`   | `c`   | Copy a secret to clipboard         |
| `wallet delete <key>` | `d`   | Remove a secret                    |

All commands accept `-f, --file <path>` to specify a custom wallet file (default: `wallet.enc`).

## How It Works

1. **`wallet init`** prompts for a master password, then creates `wallet.enc` — an encrypted JSON file.
2. Subsequent commands prompt for the same password to decrypt, read, or modify secrets.
3. Secrets are encrypted with **AES-256-GCM** using a key derived via **scrypt** with a random salt and IV per encryption.
4. The wallet file format is binary: `[16B salt][12B IV][16B auth tag][ciphertext]`.

## Build from Source

```bash
git clone <repo-url>
cd wallet
npm install
npm run build     # produces dist/index.js
```

Requires Node.js 20+, TypeScript, and tsup (included as dev deps). Tests use [Bun](https://bun.sh) — run with `bun test`.

## Security

- **AES-256-GCM** authenticated encryption
- **scrypt** key derivation (random 16-byte salt per encryption)
- Random 12-byte IV per encryption
- Secrets decrypted in memory only
- Zero plaintext written to disk

Only two runtime dependencies: `commander` (CLI parsing) and `picocolors` (terminal colors). Crypto, clipboard, and password prompts use Node.js built-in modules.
