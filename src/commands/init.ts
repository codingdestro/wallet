import { Command } from 'commander';
import { ensureWalletExists } from '../utils/wallet.js';

export function registerInitCommand(program: Command) {
  program
    .command('init')
    .description('Initialize the encrypted wallet file')
    .option('-f, --file <path>', 'Custom path for the wallet file', 'wallet.enc')
    .action(async (options) => {
      await ensureWalletExists(options.file);
    });
}
