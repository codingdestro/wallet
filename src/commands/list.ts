import { Command } from 'commander';
import { listWalletKeys } from '../utils/wallet.js';
import { userPath } from '../utils/userpath.js';

export function registerListCommand(program: Command) {
  program
    .command('list')
    .alias('l')
    .description('List all keys in the wallet')
    .option('-f, --file <path>', 'Custom path for the wallet file', 'default')
    .action(async (options) => {
      await listWalletKeys(userPath(options.file));
    });
}
