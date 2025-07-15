import { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import axios from 'axios';
import { loadStripe } from '@stripe/stripe-js';
import { Content as ContentType } from '@bits/shared';
import { importKey, decryptData, decodeIv } from '@bits/shared';
import { useAuthStore } from '../store';

const stripePromise = loadStripe(import.meta.env.VITE_STRIPE_PUBLISHABLE_KEY || '');

export default function Content() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { isAuthenticated } = useAuthStore();
  const [content, setContent] = useState<ContentType | null>(null);
  const [isPurchased, setIsPurchased] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [isPurchasing, setIsPurchasing] = useState(false);
  const [decryptedUrl, setDecryptedUrl] = useState<string | null>(null);
  const [error, setError] = useState('');

  useEffect(() => {
    if (id) {
      fetchContent();
    }
  }, [id]);

  const fetchContent = async () => {
    try {
      const { data } = await axios.get<ContentType>(`/api/content/${id}`);
      setContent(data);

      if (isAuthenticated) {
        checkPurchaseStatus();
      }
    } catch (error) {
      console.error('Failed to fetch content:', error);
      setError('Content not found');
    } finally {
      setIsLoading(false);
    }
  };

  const checkPurchaseStatus = async () => {
    try {
      const { data } = await axios.get(`/api/content/${id}/key`);
      if (data.key) {
        setIsPurchased(true);
        await decryptContent(data.key, data.iv);
      }
    } catch (error) {
      // Not purchased yet
    }
  };

  const decryptContent = async (keyString: string, ivString: string) => {
    if (!content) return;

    try {
      const response = await fetch(content.encryptedUrl);
      const encryptedData = await response.arrayBuffer();

      const key = await importKey(keyString);
      const iv = decodeIv(ivString);

      const decrypted = await decryptData(encryptedData, key, iv);
      const blob = new Blob([decrypted]);
      const url = URL.createObjectURL(blob);

      setDecryptedUrl(url);
    } catch (error) {
      console.error('Failed to decrypt content:', error);
      setError('Failed to decrypt content');
    }
  };

  const handlePurchase = async () => {
    if (!isAuthenticated) {
      navigate('/login');
      return;
    }

    setIsPurchasing(true);
    setError('');

    try {
      await axios.post('/api/purchase/intent', {
        contentId: id
      });

      const stripe = await stripePromise;
      if (!stripe) {
        throw new Error('Stripe failed to load');
      }

      // In a real app, you'd use Stripe Elements here
      // For MVP, we'll simulate a successful payment
      await axios.post('/api/purchase/confirm', {
        contentId: id,
        paymentIntentId: 'simulated_payment_' + Date.now()
      });

      await checkPurchaseStatus();
    } catch (error: any) {
      console.error('Purchase failed:', error);
      setError(error.response?.data?.error || 'Purchase failed. Please try again.');
    } finally {
      setIsPurchasing(false);
    }
  };

  if (isLoading) {
    return (
      <div className="flex justify-center items-center min-h-[calc(100vh-64px)]">
        <div className="text-gray-500">Loading...</div>
      </div>
    );
  }

  if (!content) {
    return (
      <div className="flex justify-center items-center min-h-[calc(100vh-64px)]">
        <div className="text-gray-500">Content not found</div>
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto px-4 py-8">
      <div className="bg-white rounded-lg shadow-lg overflow-hidden">
        {content.previewUrl && !isPurchased && (
          <img src={content.previewUrl} alt={content.title} className="w-full h-96 object-cover" />
        )}

        {isPurchased && decryptedUrl && (
          <div className="w-full">
            {content.encryptedUrl.includes('.mp4') || content.encryptedUrl.includes('.webm') ? (
              <video controls className="w-full">
                <source src={decryptedUrl} type="video/mp4" />
                Your browser does not support the video tag.
              </video>
            ) : content.encryptedUrl.match(/\.(jpg|jpeg|png|gif|webp)$/i) ? (
              <img src={decryptedUrl} alt={content.title} className="w-full" />
            ) : (
              <div className="p-8 text-center">
                <a
                  href={decryptedUrl}
                  download={content.title}
                  className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700"
                >
                  Download File
                </a>
              </div>
            )}
          </div>
        )}

        <div className="p-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-4">{content.title}</h1>
          <p className="text-gray-700 mb-6">{content.description}</p>

          {!isPurchased && content.priceCents && (
            <div className="border-t pt-6">
              <div className="flex items-center justify-between mb-4">
                <span className="text-2xl font-bold text-gray-900">
                  ${(content.priceCents / 100).toFixed(2)}
                </span>
                <span className="text-sm text-gray-500">One-time purchase</span>
              </div>

              {error && (
                <div className="mb-4 rounded-md bg-red-50 p-4">
                  <p className="text-sm text-red-800">{error}</p>
                </div>
              )}

              <button
                onClick={handlePurchase}
                disabled={isPurchasing}
                className="w-full flex justify-center py-3 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isPurchasing ? 'Processing...' : 'Purchase Access'}
              </button>
            </div>
          )}

          {isPurchased && (
            <div className="border-t pt-6">
              <div className="rounded-md bg-green-50 p-4">
                <p className="text-sm text-green-800">You have purchased this content. Enjoy!</p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
