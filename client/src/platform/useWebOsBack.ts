import { useEffect } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';

const WEBOS_BACK_KEYCODE = 461;

function isBackKey(event: KeyboardEvent): boolean {
  return (
    event.keyCode === WEBOS_BACK_KEYCODE ||
    event.key === 'Backspace' ||
    event.key === 'GoBack' ||
    event.key === 'BrowserBack'
  );
}

/** Handle LG Magic Remote Back with disableBackHistoryAPI: true. */
export function useWebOsBack(): void {
  const navigate = useNavigate();
  const location = useLocation();

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (!isBackKey(event)) {
        return;
      }
      // Player route has its own back handler.
      if (location.pathname.startsWith('/play/')) {
        return;
      }
      event.preventDefault();
      event.stopPropagation();

      const atHome = location.pathname === '/' || location.pathname === '';
      if (atHome) {
        window.PalmSystem?.platformBack?.();
        return;
      }
      navigate(-1);
    };

    window.addEventListener('keydown', onKeyDown, true);
    return () => window.removeEventListener('keydown', onKeyDown, true);
  }, [location.pathname, navigate]);
}
