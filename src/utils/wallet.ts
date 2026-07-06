import { existsSync, writeFileSync, readFileSync } from 'fs';
import { encryptBuffer, decryptBuffer } from './crypto.js';
import { promptPassword } from './prompts.js';
import { copyToClipboard } from './clipboard.js';
import pc from 'picocolors';

/**
 * Checks if the wallet file exists. If it does not, prompts the user
 * to enter and confirm a password (masked with '*') to create a new wallet.
 * @param walletPath Path to the wallet file (default: 'wallet.enc')
 * @returns boolean indicating if the wallet file exists or was successfully created
 */
export async function ensureWalletExists(walletPath = 'wallet.enc'): Promise<boolean> {
  if (existsSync(walletPath)) {
    return true;
  }

  console.log(pc.bold(pc.cyan('\n=== Wallet Initializer ===')));

  let password = '';
  let confirmedPassword = '';

  while (true) {
    let cancelled = false;

    while (true) {
      const pwd = await promptPassword(pc.cyan('Create a password for your wallet: '));
      if (pwd === null) {
        cancelled = true;
        break;
      }
      if (!pwd) {
        console.log(pc.red('Password cannot be empty.'));
        continue;
      }
      if (pwd.length < 4) {
        console.log(pc.red('Password must be at least 4 characters.'));
        continue;
      }
      password = pwd;
      break;
    }

    if (cancelled) {
      console.log(pc.yellow('Wallet initialization cancelled.'));
      return false;
    }

    while (true) {
      const confirmPwd = await promptPassword(pc.cyan('Confirm your password: '));
      if (confirmPwd === null) {
        cancelled = true;
        break;
      }
      if (!confirmPwd) {
        console.log(pc.red('Confirmation password cannot be empty.'));
        continue;
      }
      confirmedPassword = confirmPwd;
      break;
    }

    if (cancelled) {
      console.log(pc.yellow('Wallet initialization cancelled.'));
      return false;
    }

    if (password === confirmedPassword) {
      break;
    } else {
      console.log(pc.red('Passwords do not match. Please try again.\n'));
    }
  }

  console.log(pc.yellow('Initializing wallet file...'));

  try {
    // Construct default empty wallet in memory
    const walletData = { version: '1.0.0', entries: {} };
    const buffer = Buffer.from(JSON.stringify(walletData), 'utf-8');

    // Encrypt in memory and write directly to disk
    const encrypted = encryptBuffer(buffer, password);
    writeFileSync(walletPath, encrypted);

    console.log(pc.green('Wallet initialized successfully.'));
    console.log(pc.green(`Created encrypted wallet file: ${walletPath}`));
    return true;
  } catch (err: any) {
    console.error(pc.red(`Wallet initialization failed: ${err.message}`));
    return false;
  }
}

/**
 * Prompts user to decrypt the wallet, input a masked value for the key,
 * and saves the key-value pair into the wallet.
 * @param key The key to add/modify
 * @param walletPath Path to the wallet file (default: 'wallet.enc')
 */
export async function addWalletKey(key: string, walletPath = 'wallet.enc'): Promise<boolean> {
  const exists = await ensureWalletExists(walletPath);
  if (!exists) {
    return false;
  }

  const password = await promptPassword(pc.cyan('Enter wallet password to decrypt: '));
  if (password === null) {
    console.log(pc.yellow('Cancelled.'));
    return false;
  }
  if (!password) {
    console.error(pc.red('Password cannot be empty.'));
    return false;
  }

  let walletData: { version: string; entries: Record<string, string> };

  try {
    // Read and decrypt in memory
    const encryptedData = readFileSync(walletPath);
    const decrypted = decryptBuffer(encryptedData, password);
    const rawContent = decrypted.toString('utf-8');
    walletData = JSON.parse(rawContent);
  } catch (err) {
    console.error(pc.red('Incorrect password or corrupted wallet file.'));
    return false;
  }

  const value = await promptPassword(pc.cyan(`Enter value for key "${key}": `));
  if (value === null) {
    console.log(pc.yellow('Cancelled.'));
    return false;
  }
  if (!value) {
    console.error(pc.red('Value cannot be empty.'));
    return false;
  }

  // Safeguard against legacy array format or missing entries
  if (!walletData.entries || Array.isArray(walletData.entries)) {
    walletData.entries = {};
  }
  walletData.entries[key] = value;

  console.log(pc.yellow('Saving wallet...'));

  try {
    // Serialize, encrypt in memory and write directly to disk
    const buffer = Buffer.from(JSON.stringify(walletData), 'utf-8');
    const encrypted = encryptBuffer(buffer, password);
    writeFileSync(walletPath, encrypted);

    console.log(pc.green(`Successfully added key "${key}" to ${walletPath}.`));
    return true;
  } catch (err: any) {
    console.error(pc.red(`Failed to save wallet: ${err.message}`));
    return false;
  }
}

/**
 * Prompts user to decrypt the wallet, and lists all keys stored in it.
 * @param walletPath Path to the wallet file (default: 'wallet.enc')
 */
export async function listWalletKeys(walletPath = 'wallet.enc'): Promise<boolean> {
  const exists = existsSync(walletPath);
  if (!exists) {
    console.error(pc.red(`Wallet file "${walletPath}" does not exist.`));
    return false;
  }

  const password = await promptPassword(pc.cyan('Enter wallet password to decrypt: '));
  if (password === null) {
    console.log(pc.yellow('Cancelled.'));
    return false;
  }
  if (!password) {
    console.error(pc.red('Password cannot be empty.'));
    return false;
  }

  let walletData: { version: string; entries: Record<string, string> };

  try {
    // Read and decrypt in memory
    const encryptedData = readFileSync(walletPath);
    const decrypted = decryptBuffer(encryptedData, password);
    const rawContent = decrypted.toString('utf-8');
    walletData = JSON.parse(rawContent);
  } catch (err) {
    console.error(pc.red('Incorrect password or corrupted wallet file.'));
    return false;
  }

  const keys = Object.keys(walletData.entries || {});
  if (keys.length === 0) {
    console.log(pc.yellow('No keys found in wallet.'));
  } else {
    for (const key of keys) {
      console.log(key);
    }
  }

  return true;
}

/**
 * Prompts user to decrypt the wallet, finds the value of the key,
 * and copies it to the system clipboard.
 * @param key The key to copy
 * @param walletPath Path to the wallet file (default: 'wallet.enc')
 */
export async function copyWalletValue(key: string, walletPath = 'wallet.enc'): Promise<boolean> {
  const exists = existsSync(walletPath);
  if (!exists) {
    console.error(pc.red(`Wallet file "${walletPath}" does not exist.`));
    return false;
  }

  const password = await promptPassword(pc.cyan('Enter wallet password to decrypt: '));
  if (password === null) {
    console.log(pc.yellow('Cancelled.'));
    return false;
  }
  if (!password) {
    console.error(pc.red('Password cannot be empty.'));
    return false;
  }

  let walletData: { version: string; entries: Record<string, string> };

  try {
    // Read and decrypt in memory
    const encryptedData = readFileSync(walletPath);
    const decrypted = decryptBuffer(encryptedData, password);
    const rawContent = decrypted.toString('utf-8');
    walletData = JSON.parse(rawContent);
  } catch (err) {
    console.error(pc.red('Incorrect password or corrupted wallet file.'));
    return false;
  }

  const entries = walletData.entries || {};
  if (!(key in entries)) {
    console.error(pc.red(`Key "${key}" not found in wallet.`));
    return false;
  }

  try {
    // Zero-dependency native copy utility
    await copyToClipboard(entries[key]);
    console.log(pc.green(`Copied value of "${key}" to clipboard.`));
    return true;
  } catch (err: any) {
    console.error(pc.red(`Failed to copy to clipboard: ${err.message}`));
    return false;
  }
}

/**
 * Prompts user to decrypt the wallet, deletes the key from entries,
 * and encrypts the updated entries back to the file.
 * @param key The key to delete
 * @param walletPath Path to the wallet file (default: 'wallet.enc')
 */
export async function deleteWalletKey(key: string, walletPath = 'wallet.enc'): Promise<boolean> {
  const exists = existsSync(walletPath);
  if (!exists) {
    console.error(pc.red(`Wallet file "${walletPath}" does not exist.`));
    return false;
  }

  const password = await promptPassword(pc.cyan('Enter wallet password to decrypt: '));
  if (password === null) {
    console.log(pc.yellow('Cancelled.'));
    return false;
  }
  if (!password) {
    console.error(pc.red('Password cannot be empty.'));
    return false;
  }

  let walletData: { version: string; entries: Record<string, string> };

  try {
    // Read and decrypt in memory
    const encryptedData = readFileSync(walletPath);
    const decrypted = decryptBuffer(encryptedData, password);
    const rawContent = decrypted.toString('utf-8');
    walletData = JSON.parse(rawContent);
  } catch (err) {
    console.error(pc.red('Incorrect password or corrupted wallet file.'));
    return false;
  }

  const entries = walletData.entries || {};
  if (!(key in entries)) {
    console.error(pc.red(`Key "${key}" not found in wallet.`));
    return false;
  }

  delete entries[key];

  console.log(pc.yellow('Saving wallet...'));

  try {
    // Serialize, encrypt in memory and write directly to disk
    const buffer = Buffer.from(JSON.stringify(walletData), 'utf-8');
    const encrypted = encryptBuffer(buffer, password);
    writeFileSync(walletPath, encrypted);

    console.log(pc.green(`Successfully deleted key "${key}" from ${walletPath}.`));
    return true;
  } catch (err: any) {
    console.error(pc.red(`Failed to save wallet: ${err.message}`));
    return false;
  }
}
