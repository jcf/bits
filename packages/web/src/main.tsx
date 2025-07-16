import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App.tsx';
import './index.css';
import axios from 'axios';

// Configure axios defaults
axios.defaults.baseURL = import.meta.env.VITE_API_URL || 'http://localhost:4444';

// Startup console message
console.log('%c🚀 Bits Web App Loaded!', 'font-size: 20px; font-weight: bold; color: #4F46E5;');
console.log('%cEncrypted content marketplace', 'font-size: 14px; color: #6366F1;');
console.log('%c================================', 'color: #6366F1;');
console.log('%cℹ️  Upload encrypted content and get paid', 'font-size: 12px;');
console.log('%c🔐 All content is encrypted client-side', 'font-size: 12px;');
console.log('%c💳 Payments via Stripe (crypto coming soon)', 'font-size: 12px;');
console.log('%c🌐 API URL: ' + axios.defaults.baseURL, 'font-size: 12px; color: #9CA3AF;');

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
