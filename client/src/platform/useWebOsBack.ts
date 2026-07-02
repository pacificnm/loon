import { useEffect } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import {
  isAppBackKey,
  shouldDeferToTextInput,
} from './keyboard';

/** Handle LG Magic Remote Back with disableBackHistoryAPI: true. */
export function useWebOsBack(): void {
  const navigate = useNavigate();
  const location = useLocation();

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (shouldDeferToTextInput(event)) {
        return;
      }
      if (!isAppBackKey(event)) {
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
