import {
  randomBytes,
  scryptSync,
  createCipheriv,
  createDecipheriv,
} from "crypto";
import { readFileSync, writeFileSync } from "fs";

const ALGORITHM = "aes-256-gcm";

/**
 * Encrypts a file using AES-256-GCM.
 * Layout of the output file: [salt (16 bytes)][iv (12 bytes)][tag (16 bytes)][encrypted data]
 */
export function encryptFile(
  input: string,
  output: string,
  password: string
) {
  const salt = randomBytes(16);
  const iv = randomBytes(12);

  const key = scryptSync(password, salt, 32);
  const cipher = createCipheriv(ALGORITHM, key, iv);
  const data = readFileSync(input);

  const encrypted = Buffer.concat([
    cipher.update(data),
    cipher.final(),
  ]);

  const authTag = cipher.getAuthTag();

  const file = Buffer.concat([
    salt,
    iv,
    authTag,
    encrypted,
  ]);

  writeFileSync(output, file);
}

/**
 * Decrypts a file encrypted with encryptFile.
 */
export function decryptFile(
  input: string,
  output: string,
  password: string
) {
  const file = readFileSync(input);

  const salt = file.subarray(0, 16);
  const iv = file.subarray(16, 28);
  const authTag = file.subarray(28, 44);
  const encrypted = file.subarray(44);

  const key = scryptSync(password, salt, 32);
  const decipher = createDecipheriv(ALGORITHM, key, iv);
  decipher.setAuthTag(authTag);

  const decrypted = Buffer.concat([
    decipher.update(encrypted),
    decipher.final(),
  ]);

  writeFileSync(output, decrypted);
}
