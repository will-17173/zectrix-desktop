const GA_MEASUREMENT_ID = 'G-80CZ0C1CZT';

export function initAnalytics() {
  if (!import.meta.env.PROD) return;

  gtag('config', GA_MEASUREMENT_ID, {
    send_page_view: false,
  });
}

export function trackPageView(path: string) {
  if (!import.meta.env.PROD) return;
  gtag('event', 'page_view', { page_path: path });
}

export function trackEvent(name: string, params?: Record<string, unknown>) {
  if (!import.meta.env.PROD) return;
  gtag('event', name, params);
}