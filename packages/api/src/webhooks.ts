import Stripe from 'stripe';
import { Request, Response } from 'express';
import { createPurchase } from './db.js';

const stripe = new Stripe(process.env.STRIPE_SECRET_KEY!, {
  apiVersion: '2023-10-16'
});

export async function handleStripeWebhook(req: Request, res: Response) {
  const sig = req.headers['stripe-signature'] as string;
  const webhookSecret = process.env.STRIPE_WEBHOOK_SECRET!;

  let event: Stripe.Event;

  try {
    event = stripe.webhooks.constructEvent(
      req.body,
      sig,
      webhookSecret
    );
  } catch (err: any) {
    console.error('Webhook signature verification failed:', err.message);
    return res.status(400).send(`Webhook Error: ${err.message}`);
  }

  // Handle the event
  switch (event.type) {
    case 'payment_intent.succeeded':
      const paymentIntent = event.data.object as Stripe.PaymentIntent;
      
      // Extract metadata
      const { contentId, buyerId } = paymentIntent.metadata;
      
      if (contentId && buyerId) {
        try {
          // Create purchase record
          await createPurchase({
            buyerId,
            contentId,
            pricePaidCents: paymentIntent.amount,
            paymentMethod: 'stripe',
            transactionId: paymentIntent.id
          });
          
          console.log(`Purchase recorded for content ${contentId} by user ${buyerId}`);
        } catch (error) {
          console.error('Failed to record purchase:', error);
          // Don't fail the webhook - Stripe will retry
        }
      }
      break;

    default:
      console.log(`Unhandled event type ${event.type}`);
  }

  res.json({ received: true });
}