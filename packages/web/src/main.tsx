import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App.tsx';
import './index.css';

// Startup console message
console.log('%c🚀 Bits Web App Loaded!', 'font-size: 20px; font-weight: bold; color: #4F46E5;');
console.log('%cEncrypted content marketplace', 'font-size: 14px; color: #6366F1;');
console.log('%c================================', 'color: #6366F1;');
console.log('%cℹ️  Upload encrypted content and get paid', 'font-size: 12px;');
console.log('%c🔐 All content is encrypted client-side', 'font-size: 12px;');
console.log('%c💳 Payments via Stripe (crypto coming soon)', 'font-size: 12px;');

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
