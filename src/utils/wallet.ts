import * as p from '@clack/prompts';
import { existsSync, writeFileSync, unlinkSync } from 'fs';
import { encryptFile } from './crypto.js';

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
    // Create temporary file with default empty wallet structure
    const tempFile = `temp_${Date.now()}.json`;
    writeFileSync(tempFile, JSON.stringify({ version: '1.0.0', entries: [] }), 'utf-8');

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
