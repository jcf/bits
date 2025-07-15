export interface User {
  id: string;
  email?: string;
  walletAddress?: string;
  createdAt: Date;
}

export interface Content {
  id: string;
  creatorId: string;
  title: string;
  description: string;
  previewUrl?: string;
  encryptedUrl: string;
  priceCents?: number;
  priceUsdc?: number;
  createdAt: Date;
}

export interface Purchase {
  id: string;
  buyerId: string;
  contentId: string;
  pricePaidCents: number;
  paymentMethod: 'stripe' | 'crypto';
  transactionId: string;
  createdAt: Date;
}

export interface EncryptionKey {
  key: string;
  iv: string;
}

export interface UploadResponse {
  uploadUrl: string;
  contentId: string;
}

export interface PaymentIntent {
  clientSecret: string;
  amount: number;
}