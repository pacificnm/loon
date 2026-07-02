import { useFocusable } from '@noriginmedia/norigin-spatial-navigation';
import type { ReactNode } from 'react';
import styles from './FocusButton.module.css';

interface FocusButtonProps {
  label: string;
  focusKey?: string;
  selected?: boolean;
  onPress: () => void;
}

export function FocusButton({ label, focusKey, selected, onPress }: FocusButtonProps) {
  const { ref, focused } = useFocusable({
    focusKey,
    onEnterPress: onPress,
  });

  const className = [
    styles.button,
    focused ? styles.focused : '',
    selected ? styles.selected : '',
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <button ref={ref} type="button" className={className} onClick={onPress}>
      {label}
    </button>
  );
}

interface FocusTileProps {
  focusKey?: string;
  className?: string;
  children: ReactNode;
  onPress?: () => void;
}

export function FocusTile({ focusKey, className, children, onPress }: FocusTileProps) {
  const { ref, focused } = useFocusable({
    focusKey,
    onEnterPress: onPress,
  });

  const classes = [styles.tile, focused ? styles.focused : '', className ?? '']
    .filter(Boolean)
    .join(' ');

  return (
    <div
      ref={ref}
      className={classes}
      role="button"
      tabIndex={-1}
      onClick={onPress}
      onKeyDown={(event) => {
        if (event.key === 'Enter') {
          onPress?.();
        }
      }}
    >
      {children}
    </div>
  );
}
