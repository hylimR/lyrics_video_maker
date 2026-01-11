import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { BrowserRouter, Routes, Route } from 'react-router-dom'
import './index.css'
import App from './App.jsx'
import PreviewPage from './pages/PreviewPage.jsx'
import ExportPage from './pages/ExportPage.jsx'
import DebugPage from './pages/DebugPage.jsx'

createRoot(document.getElementById('root')).render(
  <StrictMode>
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<App />} />
        <Route path="/preview" element={<PreviewPage />} />
        <Route path="/export" element={<ExportPage />} />
        <Route path="/debug" element={<DebugPage />} />
      </Routes>
    </BrowserRouter>
  </StrictMode>,
)
