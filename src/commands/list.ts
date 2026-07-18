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
      try {
        await listWalletKeys(userPath(options.file));
      } catch (err: any) {
        process.stderr.write(`error: ${err.message}\n`);
        process.exit(1);
      }
    });
}
