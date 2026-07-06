import { Command } from 'commander';
import pc from 'picocolors';

export function createProgram() {
  const program = new Command();

  program
    .name('wallet')
    .description('A simple Node.js CLI tool template')
    .version('1.0.0');

  program
    .command('greet')
    .description('Greet a user')
    .argument('<name>', 'Name of the user to greet')
    .option('-c, --caps', 'Capitalize the name')
    .action((name, options) => {
      const formattedName = options.caps ? name.toUpperCase() : name;
      console.log(pc.green(`Hello, ${formattedName}!`));
    });

  return program;
}
