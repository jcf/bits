import { BrowserRouter as Router, Routes, Route, Link } from 'react-router-dom';
import { useEffect } from 'react';
import { useAuthStore } from './store';
import Login from './pages/Login';
import Upload from './pages/Upload';
import Browse from './pages/Browse';
import Content from './pages/Content';
import Verify from './pages/Verify';

function App() {
  const { isAuthenticated, checkAuth, logout } = useAuthStore();

  useEffect(() => {
    checkAuth();
  }, []);

  return (
    <Router>
      <div className="min-h-screen bg-gray-100">
        <nav className="bg-white shadow-sm">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="flex justify-between h-16">
              <div className="flex">
                <Link
                  to="/"
                  className="flex items-center px-2 py-2 text-2xl font-bold text-gray-900"
                >
                  Bits
                </Link>
                <div className="ml-10 flex items-center space-x-4">
                  <Link
                    to="/"
                    className="text-gray-700 hover:text-gray-900 px-3 py-2 rounded-md text-sm font-medium"
                  >
                    Browse
                  </Link>
                  {isAuthenticated && (
                    <Link
                      to="/upload"
                      className="text-gray-700 hover:text-gray-900 px-3 py-2 rounded-md text-sm font-medium"
                    >
                      Upload
                    </Link>
                  )}
                </div>
              </div>
              <div className="flex items-center">
                {isAuthenticated ? (
                  <button
                    onClick={logout}
                    className="text-gray-700 hover:text-gray-900 px-3 py-2 rounded-md text-sm font-medium"
                  >
                    Logout
                  </button>
                ) : (
                  <Link
                    to="/login"
                    className="text-gray-700 hover:text-gray-900 px-3 py-2 rounded-md text-sm font-medium"
                  >
                    Login
                  </Link>
                )}
              </div>
            </div>
          </div>
        </nav>

        <main>
          <Routes>
            <Route path="/" element={<Browse />} />
            <Route path="/login" element={<Login />} />
            <Route path="/auth/verify" element={<Verify />} />
            <Route path="/upload" element={<Upload />} />
            <Route path="/content/:id" element={<Content />} />
          </Routes>
        </main>
      </div>
    </Router>
  );
}

export default App;
