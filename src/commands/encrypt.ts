import { Command } from 'commander';
import pc from 'picocolors';
import { encryptFile } from '../utils/crypto.js';

export function registerEncryptCommand(program: Command) {
  program
    .command('encrypt')
    .description('Encrypt a file using AES-256-GCM')
    .argument('<input>', 'Path to the input file to encrypt')
    .argument('<output>', 'Path where to write the encrypted file')
    .requiredOption('-p, --password <password>', 'Password/secret key for encryption')
    .action((input, output, options) => {
      try {
        encryptFile(input, output, options.password);
        console.log(pc.green(`Successfully encrypted ${input} to ${output}`));
      } catch (err: any) {
        console.error(pc.red(`Encryption failed: ${err.message}`));
        process.exit(1);
      }
    });
}
