# Google Analytics Integration Design

## Overview

Integrate Google Analytics into the Tauri desktop application to track user behavior, page navigation, and error events. Only enabled in production builds.

## Goals

- Track todo operations (create, complete, delete)
- Track device push actions (text, image)
- Track page routing automatically
- Track errors (API failures, unhandled exceptions)
- Zero impact on development environment

## Architecture

### File Structure

```
src/
├── lib/
│   └── analytics.ts          # GA core module (init, track functions)
├── hooks/
│   └── useAnalytics.ts       # React Hook with typed event methods
├── components/
│   └── ErrorBoundary.tsx     # Capture render errors
└── main.tsx                  # Init GA on production
```

### Data Flow

```
User Action → useAnalytics Hook → analytics.ts → gtag.js → Google Analytics
              ↓
         Error Boundary → trackError()
              ↓
         window.onerror → trackError()
```

## Implementation Details

### 1. gtag.js Setup (index.html)

Add GA script to `index.html`:

```html
<script async src="https://www.googletagmanager.com/gtag/js?id=G-XXXXXXXXXX"></script>
<script>
  window.dataLayer = window.dataLayer || [];
  function gtag(){dataLayer.push(arguments);}
  gtag('js', new Date());
</script>
```

Measurement ID `G-XXXXXXXXXX` is hardcoded — replace with your actual GA Measurement ID before deployment.

**Note:** The ID is intentionally hardcoded (not environment variable) as per user preference.

### 2. Core Module (analytics.ts)

```typescript
// src/lib/analytics.ts

const GA_MEASUREMENT_ID = 'G-XXXXXXXXXX';

export function initAnalytics() {
  if (!import.meta.env.PROD) return;
  
  gtag('config', GA_MEASUREMENT_ID, {
    send_page_view: false, // We handle manually
  });
}

export function trackPageView(path: string) {
  if (!import.meta.env.PROD) return;
  gtag('event', 'page_view', { page_path: path });
}

export function trackEvent(name: string, params?: Record<string, any>) {
  if (!import.meta.env.PROD) return;
  gtag('event', name, params);
}
```

Add TypeScript declaration for `gtag`:

```typescript
// src/types/gtag.d.ts
declare function gtag(command: 'config', id: string, config?: object): void;
declare function gtag(command: 'event', name: string, params?: object): void;
declare function gtag(command: 'js', date: Date): void;
```

### 3. React Hook (useAnalytics.ts)

```typescript
// src/hooks/useAnalytics.ts
import { trackEvent } from '@/lib/analytics';

export function useAnalytics() {
  return {
    // Todo events
    trackTodoCreate: (localId: string) => 
      trackEvent('todo_create', { local_id: localId }),
    trackTodoComplete: (localId: string) => 
      trackEvent('todo_complete', { local_id: localId }),
    trackTodoDelete: (localId: string) => 
      trackEvent('todo_delete', { local_id: localId }),
    
    // Push events
    trackPushText: (deviceId: string) => 
      trackEvent('push_text', { device_id: deviceId }),
    trackPushImage: (deviceId: string) => 
      trackEvent('push_image', { device_id: deviceId }),
    
    // Error events
    trackError: (category: string, message: string) => 
      trackEvent('error', { error_category: category, error_message: message }),
  };
}
```

### 4. Page View Tracking (App.tsx)

Use `useLocation` to track route changes:

```typescript
import { useLocation } from 'react-router-dom';
import { trackPageView } from '@/lib/analytics';

function App() {
  const location = useLocation();
  
  useEffect(() => {
    trackPageView(location.pathname);
  }, [location.pathname]);
  
  // ... rest of component
}
```

### 5. Error Tracking

**Global Error Handler:**

```typescript
// src/main.tsx
window.onerror = (message, source, lineno, colno, error) => {
  trackEvent('error', {
    error_category: 'global',
    error_message: String(message),
  });
  return false;
};

window.onunhandledrejection = (event) => {
  trackEvent('error', {
    error_category: 'promise',
    error_message: String(event.reason),
  });
};
```

**Error Boundary Component:**

```typescript
// src/components/ErrorBoundary.tsx
import { Component } from 'react';
import { trackEvent } from '@/lib/analytics';

class ErrorBoundary extends Component {
  componentDidCatch(error: Error) {
    trackEvent('error', {
      error_category: 'render',
      error_message: error.message,
    });
  }
  
  render() {
    return this.props.children;
  }
}
```

**API Error Tracking:**

In `src/lib/tauri.ts`, wrap invoke calls to catch and track errors:

```typescript
async function safeInvoke<T>(cmd: string, args?: object): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (error) {
    trackEvent('error', {
      error_category: 'api',
      error_message: String(error),
    });
    throw error;
  }
}
```

## Event Categories

| Event Name | Parameters | Trigger |
|------------|------------|---------|
| `page_view` | `page_path` | Route change |
| `todo_create` | `local_id` | Create todo |
| `todo_complete` | `local_id` | Mark todo done |
| `todo_delete` | `local_id` | Delete todo |
| `push_text` | `device_id` | Push text to device |
| `push_image` | `device_id` | Push image to device |
| `error` | `error_category`, `error_message` | Any error |

## Integration Points

| Location | Handler/Function | Event |
|----------|------------------|-------|
| `todo-list-page.tsx:111` | `handleSubmit` → `onCreateTodo` | `todo_create` |
| `todo-list-page.tsx:121` | `handleToggle` → `onToggleTodo` | `todo_complete` |
| `todo-list-page.tsx:126` | `handleDelete` → `onDeleteTodo` | `todo_delete` |
| `todo-list-page.tsx:148` | `handlePush` → `onPushTodo` | `push_text` (via todo) |
| `App.tsx:233` | `pushText` (TextTemplatesPage) | `push_text` |
| `App.tsx:249` | `pushImageTemplate` | `push_image` |
| `App.tsx:279` | `pushSketch` | `push_image` |
| `App.tsx:95` | `useLocation` change | `page_view` |
| `main.tsx` | Global error handlers | `error` |

## Testing

- Development mode: No GA calls (verify `gtag` not invoked)
- Production build: Check browser DevTools Network tab for requests to `google-analytics.com`
- Manual test each event type

## Security & Privacy

- GA Measurement ID is public (no sensitive data)
- No PII (personally identifiable information) sent
- `local_id` and `device_id` are internal identifiers, not user data

**CSP Configuration:**

Current `tauri.conf.json` has `csp: null` (no restrictions). If CSP is enabled in future, must allow:
- `https://www.googletagmanager.com`
- `https://www.google-analytics.com`

Example CSP addition:
```json
"csp": "default-src 'self'; script-src 'self' https://www.googletagmanager.com; connect-src 'self' https://www.google-analytics.com"
```