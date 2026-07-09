import { existsSync, writeFileSync, readFileSync } from 'fs';
import { encryptBuffer, decryptBuffer } from './crypto.js';
import { promptPassword } from './prompts.js';
import { copyToClipboard } from './clipboard.js';
import { logger } from './logger.js';
import pc from 'picocolors';

/**
 * Checks if the wallet file exists. If it does not, prompts the user
 * to enter and confirm a password (masked with '*') to create a new wallet.
 * @param walletPath Path to the wallet file (default: 'wallet.enc')
 * @returns boolean indicating if the wallet file exists or was successfully created
 */
export async function ensureWalletExists(walletPath: string): Promise<boolean> {
  if (existsSync(walletPath)) {
    return true;
  }

  logger.header('\nwallet init');

  let password = '';
  let confirmedPassword = '';

  while (true) {
    let cancelled = false;

    while (true) {
      const pwd = await promptPassword(pc.cyan('password: '));
      if (pwd === null) {
        cancelled = true;
        break;
      }
      if (!pwd) {
        logger.error('empty');
        continue;
      }
      if (pwd.length < 4) {
        logger.error('min 4 chars');
        continue;
      }
      password = pwd;
      break;
    }

    if (cancelled) {
      logger.warn('cancelled');
      return false;
    }

    while (true) {
      const confirmPwd = await promptPassword(pc.cyan('confirm: '));
      if (confirmPwd === null) {
        cancelled = true;
        break;
      }
      if (!confirmPwd) {
        logger.error('empty');
        continue;
      }
      confirmedPassword = confirmPwd;
      break;
    }

    if (cancelled) {
      logger.warn('cancelled');
      return false;
    }

    if (password === confirmedPassword) {
      break;
    } else {
      logger.error('mismatch\n');
    }
  }

  logger.status('initializing');

  try {
    // Construct default empty wallet in memory
    const walletData = { version: '1.0.0', entries: {} };
    const buffer = Buffer.from(JSON.stringify(walletData), 'utf-8');

    // Encrypt in memory and write directly to disk
    const encrypted = encryptBuffer(buffer, password);
    writeFileSync(walletPath, encrypted);

    logger.success('initialized');
    return true;
  } catch (err: any) {
    logger.error(`init failed: ${err.message}`);
    return false;
  }
}

/**
 * Prompts user to decrypt the wallet, input a masked value for the key,
 * and saves the key-value pair into the wallet.
 * @param key The key to add/modify
 * @param walletPath Path to the wallet file (default: 'wallet.enc')
 */
export async function addWalletKey(key: string, walletPath: string): Promise<boolean> {
  const exists = await ensureWalletExists(walletPath);
  if (!exists) {
    return false;
  }

  const password = await promptPassword(pc.cyan('password: '));
  if (password === null) {
    logger.warn('cancelled');
    return false;
  }
  if (!password) {
    logger.error('empty');
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
    logger.error('bad password or corrupted wallet');
    return false;
  }

  const value = await promptPassword(pc.cyan(`value for "${key}": `));
  if (value === null) {
    logger.warn('cancelled');
    return false;
  }
  if (!value) {
    logger.error('empty value');
    return false;
  }

  // Safeguard against legacy array format or missing entries
  if (!walletData.entries || Array.isArray(walletData.entries)) {
    walletData.entries = {};
  }
  walletData.entries[key] = value;

  logger.status('saving');

  try {
    // Serialize, encrypt in memory and write directly to disk
    const buffer = Buffer.from(JSON.stringify(walletData), 'utf-8');
    const encrypted = encryptBuffer(buffer, password);
    writeFileSync(walletPath, encrypted);

    return true;
  } catch (err: any) {
    logger.error(`save failed: ${err.message}`);
    return false;
  }
}

/**
 * Prompts user to decrypt the wallet, and lists all keys stored in it.
 * @param walletPath Path to the wallet file (default: 'wallet.enc')
 */
export async function listWalletKeys(walletPath: string): Promise<boolean> {
  const exists = existsSync(walletPath);
  if (!exists) {
    logger.error('not found');
    return false;
  }

  const password = await promptPassword(pc.cyan('password: '));
  if (password === null) {
    logger.warn('cancelled');
    return false;
  }
  if (!password) {
    logger.error('empty');
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
    logger.error('bad password or corrupted wallet');
    return false;
  }

  const keys = Object.keys(walletData.entries || {});
  if (keys.length === 0) {
    logger.warn('empty');
  } else {
    for (const key of keys) {
      logger.item(key);
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
export async function copyWalletValue(key: string, walletPath: string): Promise<boolean> {
  const exists = existsSync(walletPath);
  if (!exists) {
    logger.error(`not found: ${walletPath}`);
    return false;
  }

  const password = await promptPassword(pc.cyan('password: '));
  if (password === null) {
    logger.warn('cancelled');
    return false;
  }
  if (!password) {
    logger.error('empty');
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
    logger.error('bad password or corrupted wallet');
    return false;
  }

  const entries = walletData.entries || {};
  if (!(key in entries)) {
    logger.error(`key not found: ${key}`);
    return false;
  }

  try {
    // Zero-dependency native copy utility
    await copyToClipboard(entries[key]);
    logger.success(`copied: ${key}`);
    return true;
  } catch (err: any) {
    logger.error(`clipboard: ${err.message}`);
    return false;
  }
}

/**
 * Prompts user to decrypt the wallet, deletes the key from entries,
 * and encrypts the updated entries back to the file.
 * @param key The key to delete
 * @param walletPath Path to the wallet file (default: 'wallet.enc')
 */
export async function deleteWalletKey(key: string, walletPath: string): Promise<boolean> {
  const exists = existsSync(walletPath);
  if (!exists) {
    logger.error('not found');
    return false;
  }

  const password = await promptPassword(pc.cyan('password: '));
  if (password === null) {
    logger.warn('cancelled');
    return false;
  }
  if (!password) {
    logger.error('empty');
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
    logger.error('bad password or corrupted wallet');
    return false;
  }

  const entries = walletData.entries || {};
  if (!(key in entries)) {
    logger.error(`key not found: ${key}`);
    return false;
  }

  delete entries[key];

  logger.status('saving');

  try {
    // Serialize, encrypt in memory and write directly to disk
    const buffer = Buffer.from(JSON.stringify(walletData), 'utf-8');
    const encrypted = encryptBuffer(buffer, password);
    writeFileSync(walletPath, encrypted);

    return true;
  } catch (err: any) {
    logger.error(`save failed: ${err.message}`);
    return false;
  }
}
