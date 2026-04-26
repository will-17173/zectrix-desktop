import { trackEvent } from "../lib/analytics";

export function useAnalytics() {
  return {
    trackTodoCreate: (localId: string) =>
      trackEvent('todo_create', { local_id: localId }),
    trackTodoComplete: (localId: string) =>
      trackEvent('todo_complete', { local_id: localId }),
    trackTodoDelete: (localId: string) =>
      trackEvent('todo_delete', { local_id: localId }),
    trackPushText: (deviceId: string) =>
      trackEvent('push_text', { device_id: deviceId }),
    trackPushImage: (deviceId: string) =>
      trackEvent('push_image', { device_id: deviceId }),
    trackError: (category: string, message: string) =>
      trackEvent('error', { error_category: category, error_message: message }),
  };
}