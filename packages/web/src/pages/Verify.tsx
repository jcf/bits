import { useEffect, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import axios from 'axios';
import { useAuthStore } from '../store';

export default function Verify() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const { login } = useAuthStore();
  const [message, setMessage] = useState('Verifying your magic link...');
  const [isError, setIsError] = useState(false);

  useEffect(() => {
    const verifyToken = async () => {
      const token = searchParams.get('token');

      if (!token) {
        setMessage('Invalid magic link');
        setIsError(true);
        return;
      }

      try {
        const { data } = await axios.post('/auth/verify-magic-link', { token });
        login(data.token, data.user);
        setMessage('Success! Redirecting...');
        setTimeout(() => navigate('/'), 1500);
      } catch (error) {
        setMessage('Invalid or expired magic link');
        setIsError(true);
      }
    };

    verifyToken();
  }, [searchParams, login, navigate]);

  return (
    <div className="min-h-[calc(100vh-64px)] flex items-center justify-center">
      <div className="max-w-md w-full text-center">
        <div className={`rounded-md p-4 ${isError ? 'bg-red-50' : 'bg-blue-50'}`}>
          <p className={`text-sm ${isError ? 'text-red-800' : 'text-blue-800'}`}>{message}</p>
        </div>
        {isError && (
          <div className="mt-4">
            <a href="/login" className="text-indigo-600 hover:text-indigo-500 text-sm font-medium">
              Back to login
            </a>
          </div>
        )}
      </div>
    </div>
  );
}
