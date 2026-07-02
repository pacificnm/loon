declare global {
  interface Window {
    PalmSystem?: {
      activate: () => void;
      platformBack?: () => void;
    };
  }
}

export type WebOsRelaunchHandler = () => void;

/** Register webOS TV lifecycle hooks. Required when appinfo.json sets handlesRelaunch: true. */
export function registerWebOsLifecycle(onRelaunch: WebOsRelaunchHandler): () => void {
  const handleRelaunch = () => {
    // Platform shows the app only after the handler calls activate().
    window.PalmSystem?.activate();
    onRelaunch();
  };

  document.addEventListener('webOSRelaunch', handleRelaunch);
  return () => document.removeEventListener('webOSRelaunch', handleRelaunch);
}

export function registerVisibilityHandler(onHidden: () => void, onVisible?: () => void): () => void {
  const onChange = () => {
    if (document.hidden) {
      onHidden();
    } else {
      onVisible?.();
    }
  };

  document.addEventListener('visibilitychange', onChange);
  return () => document.removeEventListener('visibilitychange', onChange);
}
