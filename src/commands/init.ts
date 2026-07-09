import { Command } from 'commander';
import { ensureWalletExists } from '../utils/wallet.js';
import { userPath } from '../utils/userpath.js';

export function registerInitCommand(program: Command) {
  program
    .command('init')
    .description('Initialize the encrypted wallet file')
    .option('-f, --file <path>', 'Custom path for the wallet file', 'default')
    .action(async (options) => {
      await ensureWalletExists(userPath(options.file));
    });
}
