import {
  randomBytes,
  scryptSync,
  createCipheriv,
  createDecipheriv,
} from "crypto";

const ALGORITHM = "aes-256-gcm";
const SCRYPT_PARAMS = { N: 16384, r: 8, p: 1 } as const;

export function encryptBuffer(data: Buffer, password: Buffer): Buffer {
  const salt = randomBytes(16);
  const iv = randomBytes(12);

  const key = scryptSync(password, salt, 32, SCRYPT_PARAMS);
  const cipher = createCipheriv(ALGORITHM, key, iv);

  const encrypted = Buffer.concat([
    cipher.update(data),
    cipher.final(),
  ]);

  const authTag = cipher.getAuthTag();
  key.fill(0);

  return Buffer.concat([
    salt,
    iv,
    authTag,
    encrypted,
  ]);
}

export function decryptBuffer(encryptedBuffer: Buffer, password: Buffer): Buffer {
  const salt = encryptedBuffer.subarray(0, 16);
  const iv = encryptedBuffer.subarray(16, 28);
  const authTag = encryptedBuffer.subarray(28, 44);
  const encrypted = encryptedBuffer.subarray(44);

  const key = scryptSync(password, salt, 32, SCRYPT_PARAMS);
  const decipher = createDecipheriv(ALGORITHM, key, iv);
  decipher.setAuthTag(authTag);

  const result = Buffer.concat([
    decipher.update(encrypted),
    decipher.final(),
  ]);

  key.fill(0);

  return result;
}
