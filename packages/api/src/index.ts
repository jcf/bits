import express from 'express';
import cors from 'cors';
import { z } from 'zod';
import { createTables, createContent, getAllContent, getContentById, createPurchase, hasPurchased, getContentKey } from './db.js';
import { sendMagicLink, verifyMagicLink, authenticateWallet, generateAuthToken, authMiddleware } from './auth.js';
import { generateUploadUrl, getPublicUrl } from './storage.js';
import { createPaymentIntent } from './payments.js';
import { handleStripeWebhook } from './webhooks.js';
import type { User } from '@bits/shared';

const app = express();
const PORT = process.env.PORT || 3000;

app.use(cors({
  origin: process.env.WEB_URL || 'http://localhost:5173',
  credentials: true
}));

// Stripe webhook needs raw body
app.post('/webhook/stripe', express.raw({ type: 'application/json' }), handleStripeWebhook);

// All other routes need JSON parsing
app.use(express.json());

interface AuthRequest extends express.Request {
  user?: User;
}

// Initialize database
await createTables();

// Auth endpoints
app.post('/auth/magic-link', async (req, res) => {
  try {
    const { email } = z.object({ email: z.string().email() }).parse(req.body);
    await sendMagicLink(email);
    res.json({ message: 'Magic link sent' });
  } catch (error) {
    res.status(400).json({ error: 'Invalid email' });
  }
});

app.post('/auth/verify-magic-link', async (req, res) => {
  try {
    const { token } = z.object({ token: z.string() }).parse(req.body);
    const user = await verifyMagicLink(token);
    const authToken = generateAuthToken(user);
    res.json({ token: authToken, user });
  } catch (error) {
    res.status(400).json({ error: 'Invalid or expired token' });
  }
});

app.post('/auth/wallet', async (req, res) => {
  try {
    const { walletAddress } = z.object({ walletAddress: z.string() }).parse(req.body);
    const user = await authenticateWallet(walletAddress);
    const token = generateAuthToken(user);
    res.json({ token, user });
  } catch (error) {
    res.status(400).json({ error: 'Invalid wallet address' });
  }
});

// Content endpoints
app.post('/content/upload-url', authMiddleware as any, async (req: AuthRequest, res) => {
  try {
    const { fileName, contentType } = z.object({
      fileName: z.string(),
      contentType: z.string()
    }).parse(req.body);

    const { uploadUrl, key } = await generateUploadUrl(fileName, contentType);
    res.json({ uploadUrl, key });
  } catch (error) {
    res.status(400).json({ error: 'Invalid request' });
  }
});

app.post('/content', authMiddleware as any, async (req: AuthRequest, res) => {
  try {
    const data = z.object({
      title: z.string(),
      description: z.string(),
      previewKey: z.string().optional(),
      encryptedKey: z.string(),
      encryptionKey: z.string(),
      encryptionIv: z.string(),
      priceCents: z.number().optional(),
      priceUsdc: z.number().optional()
    }).parse(req.body);

    const content = await createContent({
      creatorId: req.user!.id,
      title: data.title,
      description: data.description,
      previewUrl: data.previewKey ? getPublicUrl(data.previewKey) : undefined,
      encryptedUrl: getPublicUrl(data.encryptedKey),
      encryptionKey: data.encryptionKey,
      encryptionIv: data.encryptionIv,
      priceCents: data.priceCents,
      priceUsdc: data.priceUsdc
    });

    res.json(content);
  } catch (error) {
    res.status(400).json({ error: 'Invalid content data' });
  }
});

app.get('/content', async (req, res) => {
  try {
    const content = await getAllContent();
    res.json(content);
  } catch (error) {
    res.status(500).json({ error: 'Failed to fetch content' });
  }
});

app.get('/content/:id', async (req, res) => {
  try {
    const content = await getContentById(req.params.id);
    if (!content) {
      return res.status(404).json({ error: 'Content not found' });
    }
    res.json(content);
  } catch (error) {
    res.status(500).json({ error: 'Failed to fetch content' });
  }
});

// Purchase endpoints
app.post('/purchase/intent', authMiddleware as any, async (req: AuthRequest, res) => {
  try {
    const { contentId } = z.object({ contentId: z.string().uuid() }).parse(req.body);
    
    const content = await getContentById(contentId);
    if (!content || !content.priceCents) {
      return res.status(404).json({ error: 'Content not found or not for sale' });
    }

    const alreadyPurchased = await hasPurchased(req.user!.id, contentId);
    if (alreadyPurchased) {
      return res.status(400).json({ error: 'Already purchased' });
    }

    const paymentIntent = await createPaymentIntent(
      content.priceCents,
      contentId,
      req.user!.id
    );

    res.json(paymentIntent);
  } catch (error) {
    res.status(400).json({ error: 'Invalid request' });
  }
});

app.post('/purchase/confirm', authMiddleware as any, async (req: AuthRequest, res) => {
  try {
    const { contentId, paymentIntentId } = z.object({
      contentId: z.string().uuid(),
      paymentIntentId: z.string()
    }).parse(req.body);

    const content = await getContentById(contentId);
    if (!content) {
      return res.status(404).json({ error: 'Content not found' });
    }

    await createPurchase({
      buyerId: req.user!.id,
      contentId,
      pricePaidCents: content.priceCents!,
      paymentMethod: 'stripe',
      transactionId: paymentIntentId
    });

    res.json({ success: true });
  } catch (error) {
    res.status(400).json({ error: 'Purchase failed' });
  }
});

app.get('/content/:id/key', authMiddleware as any, async (req: AuthRequest, res) => {
  try {
    const contentId = req.params.id;
    
    const purchased = await hasPurchased(req.user!.id, contentId);
    if (!purchased) {
      return res.status(403).json({ error: 'Content not purchased' });
    }

    const keys = await getContentKey(contentId);
    if (!keys) {
      return res.status(404).json({ error: 'Content not found' });
    }

    res.json(keys);
  } catch (error) {
    res.status(500).json({ error: 'Failed to fetch decryption key' });
  }
});

app.listen(PORT, () => {
  console.log(`API server running on port ${PORT}`);
});