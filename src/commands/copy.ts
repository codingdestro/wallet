import { Command } from 'commander';
import { copyWalletValue } from '../utils/wallet.js';
import { userPath } from '../utils/userpath.js';

export function registerCopyCommand(program: Command) {
  program
    .command('copy')
    .alias('c')
    .argument('<key>', 'The key whose value to copy')
    .option('-f, --file <path>', 'Custom path for the wallet file', 'default')
    .description('Copy the value of a key to the clipboard')
    .action(async (key, options) => {
      try {
        await copyWalletValue(key, userPath(options.file));
      } catch (err: any) {
        process.stderr.write(`error: ${err.message}\n`);
        process.exit(1);
      }
    });
}
