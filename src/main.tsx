import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import App from "./App";
import "./App.css";
import { initAnalytics, trackEvent } from "./lib/analytics";

// Disable default context menu
window.addEventListener("contextmenu", (e) => e.preventDefault());

// Global error handlers for analytics
window.onerror = (message) => {
  trackEvent('error', { error_category: 'global', error_message: String(message) });
  return false;
};

window.onunhandledrejection = (event) => {
  trackEvent('error', { error_category: 'promise', error_message: String(event.reason) });
};

// Initialize analytics (only in production)
initAnalytics();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <BrowserRouter>
      <App />
    </BrowserRouter>
  </React.StrictMode>,
);
