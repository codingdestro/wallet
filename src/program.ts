import { Command } from "commander";
import { registerInitCommand } from "./commands/init.js";

export function createProgram() {
  const program = new Command();

  program
    .name("wallet")
    .description("A simple Node.js CLI tool template")
    .version("1.0.0");

  // Register commands
  registerInitCommand(program);

  return program;
}
