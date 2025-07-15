import { describe, it, expect } from 'vitest';
import {
  generateKey,
  exportKey,
  importKey,
  encryptData,
  decryptData,
  encodeIv,
  decodeIv
} from './crypto.js';

describe('Crypto functions', () => {
  it('should generate, export, and import a key', async () => {
    const key = await generateKey();
    expect(key).toBeInstanceOf(CryptoKey);

    const exported = await exportKey(key);
    expect(typeof exported).toBe('string');

    const imported = await importKey(exported);
    expect(imported).toBeInstanceOf(CryptoKey);
  });

  it('should encrypt and decrypt data', async () => {
    const key = await generateKey();
    const originalData = new TextEncoder().encode('Hello, World!');

    const { encrypted, iv } = await encryptData(originalData, key);
    expect(encrypted).toBeInstanceOf(ArrayBuffer);
    expect(iv).toBeInstanceOf(Uint8Array);
    expect(iv.length).toBe(12);

    const decrypted = await decryptData(encrypted, key, iv);
    const decryptedText = new TextDecoder().decode(decrypted);
    expect(decryptedText).toBe('Hello, World!');
  });

  it('should encode and decode IV', () => {
    const iv = crypto.getRandomValues(new Uint8Array(12));
    const encoded = encodeIv(iv);
    expect(typeof encoded).toBe('string');

    const decoded = decodeIv(encoded);
    expect(decoded).toBeInstanceOf(Uint8Array);
    expect(decoded.length).toBe(12);
    expect(Array.from(decoded)).toEqual(Array.from(iv));
  });

  it('should handle full encryption workflow', async () => {
    // Generate key
    const key = await generateKey();
    const keyString = await exportKey(key);

    // Encrypt data
    const data = new TextEncoder().encode('Sensitive content');
    const { encrypted, iv } = await encryptData(data, key);
    const ivString = encodeIv(iv);

    // Simulate key transmission (would happen after payment)
    const receivedKey = await importKey(keyString);
    const receivedIv = decodeIv(ivString);

    // Decrypt data
    const decrypted = await decryptData(encrypted, receivedKey, receivedIv);
    const decryptedText = new TextDecoder().decode(decrypted);

    expect(decryptedText).toBe('Sensitive content');
  });
});
