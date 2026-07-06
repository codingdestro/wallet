import { expect, test, describe, afterAll } from "bun:test";
import { writeFileSync, readFileSync, unlinkSync } from "fs";
import { encryptFile, decryptFile } from "../../src/utils/crypto.js";

describe("Crypto Utils", () => {
  const originalFile = "temp_original.txt";
  const encryptedFile = "temp_encrypted.enc";
  const decryptedFile = "temp_decrypted.txt";
  const password = "super-secret-password";
  const secretContent = "This is a secret message!";

  test("encrypt and decrypt files successfully", () => {
    writeFileSync(originalFile, secretContent, "utf-8");

    encryptFile(originalFile, encryptedFile, password);

    const encryptedData = readFileSync(encryptedFile);
    expect(encryptedData.toString("utf-8")).not.toBe(secretContent);

    decryptFile(encryptedFile, decryptedFile, password);

    const decryptedContent = readFileSync(decryptedFile, "utf-8");
    expect(decryptedContent).toBe(secretContent);
  });

  test("decryption fails with wrong password", () => {
    writeFileSync(originalFile, secretContent, "utf-8");
    encryptFile(originalFile, encryptedFile, password);

    expect(() => {
      decryptFile(encryptedFile, decryptedFile, "wrong-password");
    }).toThrow();
  });

  afterAll(() => {
    try { unlinkSync(originalFile); } catch {}
    try { unlinkSync(encryptedFile); } catch {}
    try { unlinkSync(decryptedFile); } catch {}
  });
});
