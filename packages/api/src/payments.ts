import Stripe from 'stripe';
import { PaymentIntent } from '@bits/shared';

const stripe = new Stripe(process.env.STRIPE_SECRET_KEY!, {
  apiVersion: '2023-10-16'
});

const PLATFORM_FEE_PERCENT = 10;

export async function createPaymentIntent(
  amount: number,
  contentId: string,
  buyerId: string
): Promise<PaymentIntent> {
  const paymentIntent = await stripe.paymentIntents.create({
    amount,
    currency: 'usd',
    metadata: {
      contentId,
      buyerId,
      platformFee: Math.floor(amount * PLATFORM_FEE_PERCENT / 100)
    }
  });

  return {
    clientSecret: paymentIntent.client_secret!,
    amount
  };
}

export async function confirmPayment(paymentIntentId: string): Promise<boolean> {
  const paymentIntent = await stripe.paymentIntents.retrieve(paymentIntentId);
  return paymentIntent.status === 'succeeded';
}