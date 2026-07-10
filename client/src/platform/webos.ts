//! webOS TV platform hooks (PalmSystem / lifecycle events).

declare global {
  interface Window {
    PalmSystem?: {
      activate: () => void;
      platformBack?: () => void;
    };
    webOSSystem?: {
      activate: () => void;
      platformBack?: () => void;
    };
  }
}

function activateApp(): void {
  window.webOSSystem?.activate?.();
  window.PalmSystem?.activate?.();
}

export function exitWebOsApp(): void {
  window.webOSSystem?.platformBack?.();
  window.PalmSystem?.platformBack?.();
}

export type WebOsRelaunchHandler = () => void;

/** Register webOS TV lifecycle hooks. Required when appinfo.json sets handlesRelaunch: true. */
export function registerWebOsLifecycle(onRelaunch: WebOsRelaunchHandler): () => void {
  const handleShow = () => {
    // handlesRelaunch: true — platform waits for activate() before showing the app.
    activateApp();
    onRelaunch();
  };

  document.addEventListener('webOSRelaunch', handleShow);
  document.addEventListener('webOSLaunch', handleShow);

  // Cold start with handlesRelaunch still needs activate on some webOS builds.
  activateApp();

  return () => {
    document.removeEventListener('webOSRelaunch', handleShow);
    document.removeEventListener('webOSLaunch', handleShow);
  };
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
