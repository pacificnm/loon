import { useParams } from 'react-router-dom';
import { useServerUrl, streamUrl } from '../config';
import { VideoPlayer } from '../player/VideoPlayer';

export function PlayerPage() {
  const { slug = '' } = useParams();
  const server = useServerUrl();

  if (!server) {
    return <p>No server configured. Open Admin → Settings.</p>;
  }

  return (
    <VideoPlayer
      src={streamUrl(server, slug)}
      title={slug}
      onBack={() => window.history.back()}
    />
  );
}
