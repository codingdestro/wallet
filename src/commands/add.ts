import { Command } from 'commander';
import { addWalletKey } from '../utils/wallet.js';
import { userPath } from '../utils/userpath.js';

export function registerAddCommand(program: Command) {
  program
    .command('add')
    .alias('a')
    .description('Add a key-value pair to the wallet')
    .argument('<key>', 'The key to add')
    .option('-f, --file <path>', 'Custom path for the wallet file', 'default')
    .action(async (key, options) => {
      await addWalletKey(key, userPath(options.file));
    });
}
