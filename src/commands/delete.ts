import { Command } from 'commander';
import { deleteWalletKey } from '../utils/wallet.js';

export function registerDeleteCommand(program: Command) {
  program
    .command('delete')
    .alias('d')
    .argument('<key>', 'The key to delete')
    .option('-f, --file <path>', 'Custom path for the wallet file', 'wallet.enc')
    .description('Delete a key-value pair from the wallet')
    .action(async (key, options) => {
      await deleteWalletKey(key, options.file);
    });
}
