import { useParams } from 'react-router-dom';
import { getServerUrl, streamUrl } from '../config';
import { VideoPlayer } from '../player/VideoPlayer';

export function PlayerPage() {
  const { slug = '' } = useParams();
  const server = getServerUrl();

  return (
    <VideoPlayer
      src={streamUrl(server, slug)}
      title={slug}
      onBack={() => window.history.back()}
    />
  );
}
