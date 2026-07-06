import { Command } from 'commander';
import { copyWalletValue } from '../utils/wallet.js';

export function registerCopyCommand(program: Command) {
  program
    .command('copy')
    .alias('c')
    .argument('<key>', 'The key whose value to copy')
    .option('-f, --file <path>', 'Custom path for the wallet file', 'wallet.enc')
    .description('Copy the value of a key to the clipboard')
    .action(async (key, options) => {
      await copyWalletValue(key, options.file);
    });
}
