import { useEffect } from 'react';
import {
  registerVisibilityHandler,
  registerWebOsLifecycle,
  type WebOsRelaunchHandler,
} from './webos';

export function useWebOsLifecycle(onRelaunch: WebOsRelaunchHandler): void {
  useEffect(() => registerWebOsLifecycle(onRelaunch), [onRelaunch]);
}

export function useWebOsVisibility(onHidden: () => void, onVisible?: () => void): void {
  useEffect(() => registerVisibilityHandler(onHidden, onVisible), [onHidden, onVisible]);
}
