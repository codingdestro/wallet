import { Command } from 'commander';
import pc from 'picocolors';

export function registerGreetCommand(program: Command) {
  program
    .command('greet')
    .description('Greet a user')
    .argument('<name>', 'Name of the user to greet')
    .option('-c, --caps', 'Capitalize the name')
    .action((name, options) => {
      const formattedName = options.caps ? name.toUpperCase() : name;
      console.log(pc.green(`Hello, ${formattedName}!`));
    });
}
