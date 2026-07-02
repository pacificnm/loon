import { StrictMode, useEffect } from 'react';
import { createRoot } from 'react-dom/client';
import {
  FocusContext,
  init,
  useFocusable,
} from '@noriginmedia/norigin-spatial-navigation';
import { App } from './App';
import './theme/tokens.css';

init({
  debug: false,
  visualDebug: false,
});

function RootFocusWrapper({ children }: { children: React.ReactNode }) {
  const { ref, focusKey, focusSelf } = useFocusable({
    focusable: false,
    trackChildren: true,
    isFocusBoundary: true,
  });

  useEffect(() => {
    focusSelf();
  }, [focusSelf]);

  return (
    <FocusContext.Provider value={focusKey}>
      <div ref={ref} style={{ minHeight: '100vh' }}>
        {children}
      </div>
    </FocusContext.Provider>
  );
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <RootFocusWrapper>
      <App />
    </RootFocusWrapper>
  </StrictMode>,
);
