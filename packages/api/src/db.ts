import pg from 'pg';
import { User, Content, Purchase } from '@bits/shared';

const { Pool } = pg;

const pool = new Pool({
  connectionString: process.env.DATABASE_URL || 'postgresql://bits:please@127.0.0.1:5432/bits_dev'
});

export async function createTables() {
  await pool.query(`
    CREATE TABLE IF NOT EXISTS users (
      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
      email TEXT UNIQUE,
      wallet_address TEXT UNIQUE,
      created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    );
  `);

  await pool.query(`
    CREATE TABLE IF NOT EXISTS content (
      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
      creator_id UUID REFERENCES users(id),
      title TEXT NOT NULL,
      description TEXT,
      preview_url TEXT,
      encrypted_url TEXT NOT NULL,
      encryption_key TEXT NOT NULL,
      encryption_iv TEXT NOT NULL,
      price_cents INTEGER,
      price_usdc INTEGER,
      created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    );
  `);

  await pool.query(`
    CREATE TABLE IF NOT EXISTS purchases (
      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
      buyer_id UUID REFERENCES users(id),
      content_id UUID REFERENCES content(id),
      price_paid_cents INTEGER NOT NULL,
      payment_method TEXT NOT NULL,
      transaction_id TEXT NOT NULL,
      created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
      UNIQUE(buyer_id, content_id)
    );
  `);
}

export async function findUserByEmail(email: string): Promise<User | null> {
  const result = await pool.query('SELECT * FROM users WHERE email = $1', [email]);
  return result.rows[0] || null;
}

export async function findUserByWallet(walletAddress: string): Promise<User | null> {
  const result = await pool.query('SELECT * FROM users WHERE wallet_address = $1', [walletAddress]);
  return result.rows[0] || null;
}

export async function createUser(data: { email?: string; walletAddress?: string }): Promise<User> {
  const result = await pool.query(
    'INSERT INTO users (email, wallet_address) VALUES ($1, $2) RETURNING *',
    [data.email, data.walletAddress]
  );
  return result.rows[0];
}

export async function createContent(data: {
  creatorId: string;
  title: string;
  description: string;
  previewUrl?: string;
  encryptedUrl: string;
  encryptionKey: string;
  encryptionIv: string;
  priceCents?: number;
  priceUsdc?: number;
}): Promise<Content> {
  const result = await pool.query(
    `INSERT INTO content (
      creator_id, title, description, preview_url, encrypted_url,
      encryption_key, encryption_iv, price_cents, price_usdc
    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *`,
    [
      data.creatorId,
      data.title,
      data.description,
      data.previewUrl,
      data.encryptedUrl,
      data.encryptionKey,
      data.encryptionIv,
      data.priceCents,
      data.priceUsdc
    ]
  );
  return result.rows[0];
}

export async function getAllContent(): Promise<Content[]> {
  const result = await pool.query(
    'SELECT id, creator_id, title, description, preview_url, encrypted_url, price_cents, price_usdc, created_at FROM content ORDER BY created_at DESC'
  );
  return result.rows;
}

export async function getContentById(id: string): Promise<Content | null> {
  const result = await pool.query('SELECT * FROM content WHERE id = $1', [id]);
  return result.rows[0] || null;
}

export async function createPurchase(data: {
  buyerId: string;
  contentId: string;
  pricePaidCents: number;
  paymentMethod: 'stripe' | 'crypto';
  transactionId: string;
}): Promise<Purchase> {
  const result = await pool.query(
    `INSERT INTO purchases (
      buyer_id, content_id, price_paid_cents, payment_method, transaction_id
    ) VALUES ($1, $2, $3, $4, $5) RETURNING *`,
    [data.buyerId, data.contentId, data.pricePaidCents, data.paymentMethod, data.transactionId]
  );
  return result.rows[0];
}

export async function hasPurchased(buyerId: string, contentId: string): Promise<boolean> {
  const result = await pool.query(
    'SELECT 1 FROM purchases WHERE buyer_id = $1 AND content_id = $2',
    [buyerId, contentId]
  );
  return result.rows.length > 0;
}

export async function getContentKey(
  contentId: string
): Promise<{ key: string; iv: string } | null> {
  const result = await pool.query(
    'SELECT encryption_key, encryption_iv FROM content WHERE id = $1',
    [contentId]
  );
  if (!result.rows[0]) return null;
  return {
    key: result.rows[0].encryption_key,
    iv: result.rows[0].encryption_iv
  };
}
