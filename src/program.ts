import { Command } from 'commander';
import { registerGreetCommand } from './commands/greet.js';
import { registerEncryptCommand } from './commands/encrypt.js';
import { registerDecryptCommand } from './commands/decrypt.js';

export function createProgram() {
  const program = new Command();

  program
    .name('wallet')
    .description('A simple Node.js CLI tool template')
    .version('1.0.0');

  // Register commands
  registerGreetCommand(program);
  registerEncryptCommand(program);
  registerDecryptCommand(program);

  return program;
}
