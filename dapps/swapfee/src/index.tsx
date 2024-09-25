/**
 * @file index.tsx
 * @summary Entry point for the application. Renders the App component into the root element.
 * @author
 *   - Developer: Silvio Meneguzzo
 * @date September 20, 2024
 */


import React from 'react';
import ReactDOM from 'react-dom/client';
import { App } from './App';
import './styles/App.css';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
