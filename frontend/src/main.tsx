import '@/font-size.css'

import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'

import { App } from '@/App'

const rootElement = document.getElementById('root')
if (!rootElement) {
  throw new Error('Root element not found. The application cannot be mounted.')
}

createRoot(rootElement).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
