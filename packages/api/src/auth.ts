import jwt from 'jsonwebtoken';
import { Request, Response, NextFunction } from 'express';
import { createUser, findUserByEmail, findUserByWallet } from './db.js';
import nodemailer from 'nodemailer';
import { User } from '@bits/shared';

const JWT_SECRET = process.env.JWT_SECRET || 'dev-secret';
const MAGIC_LINK_SECRET = process.env.MAGIC_LINK_SECRET || 'dev-magic-secret';

interface AuthRequest extends Request {
  user?: User;
}

const transporter =
  process.env.AWS_ACCESS_KEY_ID && process.env.AWS_SECRET_ACCESS_KEY
    ? nodemailer.createTransport({
        host: `email-smtp.${process.env.AWS_REGION}.amazonaws.com`,
        port: 587,
        secure: false,
        auth: {
          user: process.env.AWS_ACCESS_KEY_ID,
          pass: process.env.AWS_SECRET_ACCESS_KEY
        }
      })
    : null;

export async function sendMagicLink(email: string) {
  const token = jwt.sign({ email }, MAGIC_LINK_SECRET, { expiresIn: '15m' });
  const magicLink = `${process.env.WEB_URL}/auth/verify?token=${token}`;

  if (transporter) {
    await transporter.sendMail({
      from: process.env.SES_FROM_EMAIL || 'noreply@bits-please.com',
      to: email,
      subject: 'Sign in to Bits',
      html: `
        <p>Click the link below to sign in to Bits:</p>
        <a href="${magicLink}">Sign in to Bits</a>
        <p>This link will expire in 15 minutes.</p>
      `
    });
  } else {
    // Development fallback - log to console
    console.log('=== MAGIC LINK (Email not configured) ===');
    console.log(`To: ${email}`);
    console.log(`Link: ${magicLink}`);
    console.log('=========================================');
  }
}

export async function verifyMagicLink(token: string): Promise<User> {
  try {
    const decoded = jwt.verify(token, MAGIC_LINK_SECRET) as { email: string };
    let user = await findUserByEmail(decoded.email);
    if (!user) {
      user = await createUser({ email: decoded.email });
    }
    return user;
  } catch (error) {
    throw new Error('Invalid or expired magic link');
  }
}

export async function authenticateWallet(walletAddress: string): Promise<User> {
  let user = await findUserByWallet(walletAddress);
  if (!user) {
    user = await createUser({ walletAddress });
  }
  return user;
}

export function generateAuthToken(user: User): string {
  return jwt.sign({ userId: user.id }, JWT_SECRET, { expiresIn: '30d' });
}

export async function authMiddleware(req: AuthRequest, res: Response, next: NextFunction) {
  const token = req.headers.authorization?.replace('Bearer ', '');

  if (!token) {
    return res.status(401).json({ error: 'No token provided' });
  }

  try {
    const decoded = jwt.verify(token, JWT_SECRET) as { userId: string };
    req.user = { id: decoded.userId } as User;
    next();
  } catch (error) {
    return res.status(401).json({ error: 'Invalid token' });
  }
}
