import { useEffect, useRef } from 'react';
import { isAppBackKey } from '../platform/keyboard';
import { useWebOsVisibility } from '../platform/useWebOsLifecycle';
import styles from './VideoPlayer.module.css';

interface VideoPlayerProps {
  src: string;
  title: string;
  onBack: () => void;
}

export function VideoPlayer({ src, title, onBack }: VideoPlayerProps) {
  const videoRef = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    const video = videoRef.current;
    if (!video) {
      return;
    }
    video.src = src;
    void video.play().catch(() => {
      /* autoplay policy — user can press play on TV */
    });
  }, [src]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (!isAppBackKey(event)) {
        return;
      }
      event.preventDefault();
      onBack();
    };
    window.addEventListener('keydown', onKeyDown, true);
    return () => window.removeEventListener('keydown', onKeyDown, true);
  }, [onBack]);

  useWebOsVisibility(() => {
    videoRef.current?.pause();
  });

  return (
    <div className={styles.shell}>
      <video
        ref={videoRef}
        className={styles.video}
        controls
        playsInline
        title={title}
      />
    </div>
  );
}
