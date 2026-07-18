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
      try {
        await deleteWalletKey(key, userPath(options.file));
      } catch (err: any) {
        process.stderr.write(`error: ${err.message}\n`);
        process.exit(1);
      }
    });
}
