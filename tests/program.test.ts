import { expect, test, describe, spyOn, afterAll } from "bun:test";
import { writeFileSync, readFileSync, unlinkSync } from "fs";
import { createProgram } from "../src/program.js";

describe("CLI program integration", () => {
  const originalFile = "temp_cli_original.txt";
  const encryptedFile = "temp_cli_encrypted.enc";
  const decryptedFile = "temp_cli_decrypted.txt";
  const password = "my-secret-key";
  const secretContent = "CLI integration testing!";

  test("defines metadata correctly", () => {
    const program = createProgram();
    expect(program.name()).toBe("wallet");
    expect(program.description()).toBe("A simple Node.js CLI tool template");
    expect(program.version()).toBe("1.0.0");
  });

  test("encrypt and decrypt commands function correctly via CLI", () => {
    const program = createProgram();
    const logSpy = spyOn(console, "log").mockImplementation(() => {});

    writeFileSync(originalFile, secretContent, "utf-8");

    program.parse(["node", "index.js", "encrypt", originalFile, encryptedFile, "-p", password]);
    expect(logSpy).toHaveBeenCalled();
    expect(logSpy.mock.calls[0][0]).toContain("Successfully encrypted");

    const programDecrypt = createProgram();
    programDecrypt.parse(["node", "index.js", "decrypt", encryptedFile, decryptedFile, "-p", password]);
    expect(logSpy.mock.calls[1][0]).toContain("Successfully decrypted");

    const decryptedContent = readFileSync(decryptedFile, "utf-8");
    expect(decryptedContent).toBe(secretContent);

    logSpy.mockRestore();
  });

  afterAll(() => {
    try { unlinkSync(originalFile); } catch {}
    try { unlinkSync(encryptedFile); } catch {}
    try { unlinkSync(decryptedFile); } catch {}
  });
});
