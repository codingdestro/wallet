import { Command } from 'commander';
import { changeWalletPassword } from '../utils/wallet.js';
import { userPath } from '../utils/userpath.js';

export function registerPasswdCommand(program: Command) {
  program
    .command('passwd')
    .description('Change the wallet master password')
    .option('-f, --file <path>', 'Custom path for the wallet file', 'default')
    .action(async (options) => {
      try {
        await changeWalletPassword(userPath(options.file));
      } catch (err: any) {
        process.stderr.write(`error: ${err.message}\n`);
        process.exit(1);
      }
    });
}
