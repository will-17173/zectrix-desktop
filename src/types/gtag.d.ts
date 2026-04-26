declare function gtag(command: 'config', id: string, config?: object): void;
declare function gtag(command: 'event', name: string, params?: object): void;
declare function gtag(command: 'js', date: Date): void;

interface Window {
  dataLayer: unknown[];
  gtag: typeof gtag;
}