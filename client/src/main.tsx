import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import {
  FocusContext,
  init,
  useFocusable,
} from '@noriginmedia/norigin-spatial-navigation';
import { App } from './App';
import './theme/tokens.css';

init({
  debug: import.meta.env.DEV,
  visualDebug: false,
  distanceCalculationMethod: 'center',
  useGetBoundingClientRect: true,
});

function RootFocusWrapper({ children }: { children: React.ReactNode }) {
  const { ref, focusKey } = useFocusable({
    focusable: false,
    trackChildren: true,
    isFocusBoundary: true,
    focusKey: 'root',
  });

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
