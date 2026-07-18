import { existsSync, writeFileSync, readFileSync, chmodSync, openSync, closeSync, readdirSync, unlinkSync } from 'fs';
import { dirname, basename, join } from 'path';
import { encryptBuffer, decryptBuffer } from './crypto.js';
import { promptPassword } from './prompts.js';
import { copyToClipboard } from './clipboard.js';
import { logger } from './logger.js';
import { validateKey } from './userpath.js';
import pc from 'picocolors';

const MIN_PASSWORD_LENGTH = 12
const MAX_FAILED_ATTEMPTS = 5
const FILE_MODE = 0o600

let failedAttempts = 0

function sanitizePassword(password: string): Buffer {
  const buf = Buffer.from(password, 'utf-8')
  return buf
}

function clearBuffer(buf: Buffer): void {
  buf.fill(0)
}

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

function isProcessRunning(pid: number): boolean {
  try {
    process.kill(pid, 0)
    return true
  } catch {
    return false
  }
}

function acquireLock(walletPath: string): { release: () => void } {
  const lockPath = walletPath + '.lock.' + process.pid
  const base = basename(walletPath)

  // Clean up locks from dead processes
  try {
    const dir = dirname(walletPath)
    const files = readdirSync(dir)
    for (const f of files) {
      if (f.startsWith(base + '.lock.')) {
        const pidStr = f.slice((base + '.lock.').length)
        const pid = parseInt(pidStr, 10)
        if (!isNaN(pid) && !isProcessRunning(pid)) {
          try { unlinkSync(join(dir, f)) } catch {}
        }
      }
    }
  } catch {}

  try {
    const fd = openSync(lockPath, 'wx')
    closeSync(fd)
  } catch {
    throw new Error('locked')
  }
  return {
    release: () => {
      try { unlinkSync(lockPath) } catch {}
    }
  }
}

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
      if (pwd.length < MIN_PASSWORD_LENGTH) {
        logger.error(`min ${MIN_PASSWORD_LENGTH} chars`);
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

  const pwdBuf = sanitizePassword(password)

  try {
    const walletData = { version: '1.0.0', entries: {} };
    const buffer = Buffer.from(JSON.stringify(walletData), 'utf-8');

    const encrypted = encryptBuffer(buffer, pwdBuf);
    writeFileSync(walletPath, encrypted, { mode: FILE_MODE });
    chmodSync(walletPath, FILE_MODE);

    logger.success('initialized');
    return true;
  } catch (err: any) {
    logger.error(`init failed: ${err.message}`);
    return false;
  } finally {
    clearBuffer(pwdBuf)
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

  validateKey(key)

  const password = await promptPassword(pc.cyan('password: '));
  if (password === null) {
    logger.warn('cancelled');
    return false;
  }
  if (!password) {
    logger.error('empty');
    return false;
  }

  const pwdBuf = sanitizePassword(password)

  let lock: { release: () => void } | undefined

  try {
    try {
      lock = acquireLock(walletPath)
    } catch {
      logger.error('locked')
      return false
    }

    let walletData: { version: string; entries: Record<string, string> };

    try {
      const encryptedData = readFileSync(walletPath);
      const decrypted = decryptBuffer(encryptedData, pwdBuf);
      const rawContent = decrypted.toString('utf-8');
      walletData = JSON.parse(rawContent);
      failedAttempts = 0
    } catch (err) {
      failedAttempts++
      if (failedAttempts >= MAX_FAILED_ATTEMPTS) {
        logger.error('too many failed attempts, try again later')
        process.exit(1)
      }
      const delay = Math.pow(2, failedAttempts) * 500
      logger.error('bad password or corrupted wallet')
      await sleep(delay)
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

    if (!walletData.entries || Array.isArray(walletData.entries)) {
      walletData.entries = {};
    }
    walletData.entries[key] = value;

    logger.status('saving');

    const buffer = Buffer.from(JSON.stringify(walletData), 'utf-8');
    const encrypted = encryptBuffer(buffer, pwdBuf);
    writeFileSync(walletPath, encrypted, { mode: FILE_MODE });
    chmodSync(walletPath, FILE_MODE);

    return true;
  } catch (err: any) {
    logger.error(`save failed: ${err.message}`);
    return false;
  } finally {
    clearBuffer(pwdBuf)
    lock?.release();
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

  const pwdBuf = sanitizePassword(password)

  try {
    let walletData: { version: string; entries: Record<string, string> };

    try {
      const encryptedData = readFileSync(walletPath);
      const decrypted = decryptBuffer(encryptedData, pwdBuf);
      const rawContent = decrypted.toString('utf-8');
      walletData = JSON.parse(rawContent);
      failedAttempts = 0
    } catch (err) {
      failedAttempts++
      if (failedAttempts >= MAX_FAILED_ATTEMPTS) {
        logger.error('too many failed attempts, try again later')
        process.exit(1)
      }
      const delay = Math.pow(2, failedAttempts) * 500
      logger.error('bad password or corrupted wallet')
      await sleep(delay)
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
  } finally {
    clearBuffer(pwdBuf)
  }
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

  const pwdBuf = sanitizePassword(password)

  try {
    let walletData: { version: string; entries: Record<string, string> };

    try {
      const encryptedData = readFileSync(walletPath);
      const decrypted = decryptBuffer(encryptedData, pwdBuf);
      const rawContent = decrypted.toString('utf-8');
      walletData = JSON.parse(rawContent);
      failedAttempts = 0
    } catch (err) {
      failedAttempts++
      if (failedAttempts >= MAX_FAILED_ATTEMPTS) {
        logger.error('too many failed attempts, try again later')
        process.exit(1)
      }
      const delay = Math.pow(2, failedAttempts) * 500
      logger.error('bad password or corrupted wallet')
      await sleep(delay)
      return false;
    }

    const entries = walletData.entries || {};
    if (!(key in entries)) {
      logger.error(`key not found: ${key}`);
      return false;
    }

    const secret = entries[key]

    try {
      await copyToClipboard(secret);
      logger.success(`copied: ${key}`);
      return true;
    } catch (err: any) {
      logger.error(`clipboard: ${err.message}`);
      return false;
    }
  } finally {
    clearBuffer(pwdBuf)
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

  const pwdBuf = sanitizePassword(password)
  let lock: { release: () => void } | undefined

  try {
    try {
      lock = acquireLock(walletPath)
    } catch {
      logger.error('locked')
      return false
    }

    let walletData: { version: string; entries: Record<string, string> };

    try {
      const encryptedData = readFileSync(walletPath);
      const decrypted = decryptBuffer(encryptedData, pwdBuf);
      const rawContent = decrypted.toString('utf-8');
      walletData = JSON.parse(rawContent);
      failedAttempts = 0
    } catch (err) {
      failedAttempts++
      if (failedAttempts >= MAX_FAILED_ATTEMPTS) {
        logger.error('too many failed attempts, try again later')
        process.exit(1)
      }
      const delay = Math.pow(2, failedAttempts) * 500
      logger.error('bad password or corrupted wallet')
      await sleep(delay)
      return false;
    }

    const entries = walletData.entries || {};
    if (!(key in entries)) {
      logger.error(`key not found: ${key}`);
      return false;
    }

    delete entries[key];

    logger.status('saving');

    const buffer = Buffer.from(JSON.stringify(walletData), 'utf-8');
    const encrypted = encryptBuffer(buffer, pwdBuf);
    writeFileSync(walletPath, encrypted, { mode: FILE_MODE });
    chmodSync(walletPath, FILE_MODE);

    return true;
  } catch (err: any) {
    logger.error(`save failed: ${err.message}`);
    return false;
  } finally {
    clearBuffer(pwdBuf)
    lock?.release();
  }
}
