import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './i18n'
import App from './App'
import { installRuntimeErrorLogging } from './services/tauriApi'

installRuntimeErrorLogging()

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
