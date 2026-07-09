import { Command } from "commander";
import { registerInitCommand } from "./commands/init.js";
import { registerAddCommand } from "./commands/add.js";
import { registerListCommand } from "./commands/list.js";
import { registerCopyCommand } from "./commands/copy.js";
import { registerDeleteCommand } from "./commands/delete.js";
import { setLogLevel } from "./utils/logger.js";

export function createProgram() {
  const program = new Command();

  program
    .name("wallet")
    .description("A Secret management tool. ")
    .version("0.1.0")
    .option("-v, --verbose", "Enable verbose logging")
    .option("-q, --quiet", "Silence all output except errors")
    .hook("preAction", (cmd) => {
      const opts = cmd.opts();
      if (opts.quiet) setLogLevel("silent");
      else if (opts.verbose) setLogLevel("verbose");
    });

  // Register commands
  registerInitCommand(program);
  registerAddCommand(program);
  registerListCommand(program);
  registerCopyCommand(program);
  registerDeleteCommand(program);

  return program;
}
