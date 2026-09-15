import { useEffect, useRef, useState } from "react";
import type { ReleaseArtwork } from "../artwork";

export function HeroMedia({ artwork }: { artwork: ReleaseArtwork }) {
  const video = useRef<HTMLVideoElement>(null);
  const [failedVideo, setFailedVideo] = useState<string>();
  const [reducedMotion, setReducedMotion] = useState(false);

  useEffect(() => {
    const query = window.matchMedia("(prefers-reduced-motion: reduce)");
    const update = () => setReducedMotion(query.matches);
    update();
    query.addEventListener("change", update);
    return () => query.removeEventListener("change", update);
  }, []);

  useEffect(() => {
    const syncPlayback = () => {
      const element = video.current;
      if (!element) return;
      if (reducedMotion || document.hidden || !document.hasFocus()) {
        element.pause();
      } else {
        void element.play().catch(() => setFailedVideo(artwork.video));
      }
    };
    document.addEventListener("visibilitychange", syncPlayback);
    window.addEventListener("focus", syncPlayback);
    window.addEventListener("blur", syncPlayback);
    syncPlayback();
    return () => {
      document.removeEventListener("visibilitychange", syncPlayback);
      window.removeEventListener("focus", syncPlayback);
      window.removeEventListener("blur", syncPlayback);
    };
  }, [artwork.video, reducedMotion]);

  const useVideo = Boolean(artwork.video) && failedVideo !== artwork.video && !reducedMotion;
  return <div className="hero-media" role="img" aria-label={artwork.alt}>
    {useVideo ? <video ref={video} src={artwork.video} poster={artwork.image} muted autoPlay loop playsInline preload="metadata" onError={() => setFailedVideo(artwork.video)} style={{ objectPosition: artwork.position }} />
      : <img src={artwork.image} alt="" style={{ objectPosition: artwork.position }} />}
  </div>;
}
