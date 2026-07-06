import { Command } from "commander";
import { registerInitCommand } from "./commands/init.js";
import { registerAddCommand } from "./commands/add.js";
import { registerListCommand } from "./commands/list.js";
import { registerCopyCommand } from "./commands/copy.js";
import { registerDeleteCommand } from "./commands/delete.js";

export function createProgram() {
  const program = new Command();

  program
    .name("wallet")
    .description("A Secret management tool. ")
    .version("0.1.0");

  // Register commands
  registerInitCommand(program);
  registerAddCommand(program);
  registerListCommand(program);
  registerCopyCommand(program);
  registerDeleteCommand(program);

  return program;
}
