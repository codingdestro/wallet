import {
  randomBytes,
  scryptSync,
  createCipheriv,
  createDecipheriv,
} from "crypto";

const ALGORITHM = "aes-256-gcm";

/**
 * Encrypts a Buffer using AES-256-GCM in memory.
 * Layout of the output Buffer: [salt (16 bytes)][iv (12 bytes)][tag (16 bytes)][encrypted data]
 */
export function encryptBuffer(data: Buffer, password: string): Buffer {
  const salt = randomBytes(16);
  const iv = randomBytes(12);

  const key = scryptSync(password, salt, 32);
  const cipher = createCipheriv(ALGORITHM, key, iv);

  const encrypted = Buffer.concat([
    cipher.update(data),
    cipher.final(),
  ]);

  const authTag = cipher.getAuthTag();

  return Buffer.concat([
    salt,
    iv,
    authTag,
    encrypted,
  ]);
}

/**
 * Decrypts a Buffer in memory encrypted with encryptBuffer.
 */
export function decryptBuffer(encryptedBuffer: Buffer, password: string): Buffer {
  const salt = encryptedBuffer.subarray(0, 16);
  const iv = encryptedBuffer.subarray(16, 28);
  const authTag = encryptedBuffer.subarray(28, 44);
  const encrypted = encryptedBuffer.subarray(44);

  const key = scryptSync(password, salt, 32);
  const decipher = createDecipheriv(ALGORITHM, key, iv);
  decipher.setAuthTag(authTag);

  return Buffer.concat([
    decipher.update(encrypted),
    decipher.final(),
  ]);
}
