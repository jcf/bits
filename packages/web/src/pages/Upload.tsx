import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import axios from 'axios';
import { generateKey, exportKey, encryptData, encodeIv } from '@bits/shared';

export default function Upload() {
  const navigate = useNavigate();
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [price, setPrice] = useState('');
  const [file, setFile] = useState<File | null>(null);
  const [preview, setPreview] = useState<File | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!file) {
      setError('Please select a file to upload');
      return;
    }

    setIsLoading(true);
    setError('');

    try {
      // Generate encryption key
      const key = await generateKey();
      const keyString = await exportKey(key);

      // Encrypt the main file
      const fileBuffer = await file.arrayBuffer();
      const { encrypted, iv } = await encryptData(fileBuffer, key);
      const encryptedBlob = new Blob([encrypted], { type: file.type });

      // Get upload URLs
      const { data: mainUpload } = await axios.post('/api/content/upload-url', {
        fileName: file.name,
        contentType: file.type
      });

      let previewKey = undefined;
      if (preview) {
        const { data: previewUpload } = await axios.post('/api/content/upload-url', {
          fileName: preview.name,
          contentType: preview.type
        });
        
        // Upload preview file
        await axios.put(previewUpload.uploadUrl, preview, {
          headers: { 'Content-Type': preview.type }
        });
        
        previewKey = previewUpload.key;
      }

      // Upload encrypted file
      await axios.put(mainUpload.uploadUrl, encryptedBlob, {
        headers: { 'Content-Type': file.type }
      });

      // Create content record
      const { data: content } = await axios.post('/api/content', {
        title,
        description,
        previewKey,
        encryptedKey: mainUpload.key,
        encryptionKey: keyString,
        encryptionIv: encodeIv(iv),
        priceCents: Math.round(parseFloat(price) * 100)
      });

      navigate(`/content/${content.id}`);
    } catch (error) {
      console.error('Upload failed:', error);
      setError('Failed to upload content. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="max-w-2xl mx-auto mt-8 px-4">
      <h1 className="text-3xl font-bold text-gray-900 mb-8">Upload Content</h1>
      
      <form onSubmit={handleSubmit} className="space-y-6">
        <div>
          <label htmlFor="title" className="block text-sm font-medium text-gray-700">
            Title
          </label>
          <input
            id="title"
            type="text"
            required
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
          />
        </div>

        <div>
          <label htmlFor="description" className="block text-sm font-medium text-gray-700">
            Description
          </label>
          <textarea
            id="description"
            rows={4}
            required
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
          />
        </div>

        <div>
          <label htmlFor="price" className="block text-sm font-medium text-gray-700">
            Price (USD)
          </label>
          <input
            id="price"
            type="number"
            step="0.01"
            min="0.01"
            required
            value={price}
            onChange={(e) => setPrice(e.target.value)}
            className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
            placeholder="9.99"
          />
        </div>

        <div>
          <label htmlFor="file" className="block text-sm font-medium text-gray-700">
            Content File
          </label>
          <input
            id="file"
            type="file"
            required
            onChange={(e) => setFile(e.target.files?.[0] || null)}
            className="mt-1 block w-full text-sm text-gray-500 file:mr-4 file:py-2 file:px-4 file:rounded-md file:border-0 file:text-sm file:font-semibold file:bg-indigo-50 file:text-indigo-700 hover:file:bg-indigo-100"
          />
        </div>

        <div>
          <label htmlFor="preview" className="block text-sm font-medium text-gray-700">
            Preview Image (optional)
          </label>
          <input
            id="preview"
            type="file"
            accept="image/*"
            onChange={(e) => setPreview(e.target.files?.[0] || null)}
            className="mt-1 block w-full text-sm text-gray-500 file:mr-4 file:py-2 file:px-4 file:rounded-md file:border-0 file:text-sm file:font-semibold file:bg-indigo-50 file:text-indigo-700 hover:file:bg-indigo-100"
          />
        </div>

        {error && (
          <div className="rounded-md bg-red-50 p-4">
            <p className="text-sm text-red-800">{error}</p>
          </div>
        )}

        <button
          type="submit"
          disabled={isLoading}
          className="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isLoading ? 'Encrypting and uploading...' : 'Upload Content'}
        </button>
      </form>
    </div>
  );
}