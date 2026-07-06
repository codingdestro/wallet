import { Command } from 'commander';
import { listWalletKeys } from '../utils/wallet.js';

export function registerListCommand(program: Command) {
  program
    .command('list')
    .alias('l')
    .description('List all keys in the wallet')
    .option('-f, --file <path>', 'Custom path for the wallet file', 'wallet.enc')
    .action(async (options) => {
      await listWalletKeys(options.file);
    });
}
