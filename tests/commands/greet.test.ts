import { expect, test, describe, spyOn } from "bun:test";
import { createProgram } from "../../src/program.js";

describe("Greet Command", () => {
  test("greet command prints correct message", () => {
    const program = createProgram();
    const logSpy = spyOn(console, "log").mockImplementation(() => {});
    
    program.parse(["node", "index.js", "greet", "Alice"]);
    
    expect(logSpy).toHaveBeenCalled();
    const calledWith = logSpy.mock.calls[0][0];
    expect(calledWith).toContain("Hello, Alice!");

    logSpy.mockRestore();
  });

  test("greet command with caps option capitalizes name", () => {
    const program = createProgram();
    const logSpy = spyOn(console, "log").mockImplementation(() => {});
    
    program.parse(["node", "index.js", "greet", "Alice", "--caps"]);
    
    expect(logSpy).toHaveBeenCalled();
    const calledWith = logSpy.mock.calls[0][0];
    expect(calledWith).toContain("Hello, ALICE!");

    logSpy.mockRestore();
  });
});
