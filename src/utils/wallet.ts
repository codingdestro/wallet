import * as p from '@clack/prompts';
import { existsSync, writeFileSync, readFileSync, unlinkSync } from 'fs';
import { encryptFile, decryptFile } from './crypto.js';

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

  p.intro('Wallet Initializer');

  let password = '';
  let confirmedPassword = '';

  while (true) {
    const pwdInput = await p.password({
      message: 'Create a password for your wallet',
      validate: (value) => {
        if (!value) return 'Password cannot be empty';
        if (value.length < 4) return 'Password must be at least 4 characters';
      }
    });

    if (p.isCancel(pwdInput)) {
      p.cancel('Wallet initialization cancelled.');
      return false;
    }

    password = pwdInput as string;

    const confirmInput = await p.password({
      message: 'Confirm your password',
      validate: (value) => {
        if (!value) return 'Confirmation password cannot be empty';
      }
    });

    if (p.isCancel(confirmInput)) {
      p.cancel('Wallet initialization cancelled.');
      return false;
    }

    confirmedPassword = confirmInput as string;

    if (password === confirmedPassword) {
      break;
    } else {
      p.log.error('Passwords do not match. Please try again.');
    }
  }

  const spinner = p.spinner();
  spinner.start('Initializing wallet file...');

  try {
    // Create temporary file with default empty wallet structure (Record-based entries)
    const tempFile = `temp_${Date.now()}.json`;
    writeFileSync(tempFile, JSON.stringify({ version: '1.0.0', entries: {} }), 'utf-8');

    // Encrypt to target walletPath
    encryptFile(tempFile, walletPath, password);

    // Clean up temporary file
    unlinkSync(tempFile);

    spinner.stop('Wallet initialized successfully.');
    p.outro(`Created encrypted wallet file: ${walletPath}`);
    return true;
  } catch (err: any) {
    spinner.stop('Wallet initialization failed.');
    p.log.error(`Error: ${err.message}`);
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

  const pwdInput = await p.password({
    message: 'Enter wallet password to decrypt',
    validate: (value) => {
      if (!value) return 'Password cannot be empty';
    }
  });

  if (p.isCancel(pwdInput)) {
    p.cancel('Cancelled.');
    return false;
  }

  const password = pwdInput as string;

  let walletData: { version: string; entries: Record<string, string> };
  const tempDecrypted = `temp_dec_add_${Date.now()}.json`;

  try {
    decryptFile(walletPath, tempDecrypted, password);
    const rawContent = readFileSync(tempDecrypted, 'utf-8');
    walletData = JSON.parse(rawContent);
    unlinkSync(tempDecrypted);
  } catch (err) {
    try { unlinkSync(tempDecrypted); } catch {}
    p.log.error('Incorrect password or corrupted wallet file.');
    return false;
  }

  const valInput = await p.password({
    message: `Enter value for key "${key}"`,
    validate: (value) => {
      if (!value) return 'Value cannot be empty';
    }
  });

  if (p.isCancel(valInput)) {
    p.cancel('Cancelled.');
    return false;
  }

  const value = valInput as string;

  // Safeguard against legacy array format or missing entries
  if (!walletData.entries || Array.isArray(walletData.entries)) {
    walletData.entries = {};
  }
  walletData.entries[key] = value;

  const spinner = p.spinner();
  spinner.start('Saving wallet...');

  try {
    const tempFile = `temp_enc_add_${Date.now()}.json`;
    writeFileSync(tempFile, JSON.stringify(walletData), 'utf-8');
    encryptFile(tempFile, walletPath, password);
    unlinkSync(tempFile);
    spinner.stop(`Successfully added key "${key}" to ${walletPath}.`);
    return true;
  } catch (err: any) {
    spinner.stop('Failed to save wallet.');
    p.log.error(`Error: ${err.message}`);
    return false;
  }
}
