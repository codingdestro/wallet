import { Command } from 'commander';
import pc from 'picocolors';
import { decryptFile } from '../utils/crypto.js';

export function registerDecryptCommand(program: Command) {
  program
    .command('decrypt')
    .description('Decrypt a file encrypted with AES-256-GCM')
    .argument('<input>', 'Path to the encrypted file')
    .argument('<output>', 'Path where to write the decrypted file')
    .requiredOption('-p, --password <password>', 'Password/secret key for decryption')
    .action((input, output, options) => {
      try {
        decryptFile(input, output, options.password);
        console.log(pc.green(`Successfully decrypted ${input} to ${output}`));
      } catch (err: any) {
        console.error(pc.red(`Decryption failed: ${err.message}`));
        process.exit(1);
      }
    });
}
