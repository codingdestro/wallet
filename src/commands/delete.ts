import { Command } from 'commander';
import { deleteWalletKey } from '../utils/wallet.js';
import { userPath } from '../utils/userpath.js';

export function registerDeleteCommand(program: Command) {
  program
    .command('delete')
    .alias('d')
    .argument('<key>', 'The key to delete')
    .option('-f, --file <path>', 'Custom path for the wallet file', 'default')
    .description('Delete a key-value pair from the wallet')
    .action(async (key, options) => {
      await deleteWalletKey(key, userPath(options.file));
    });
}
