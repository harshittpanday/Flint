import defaultHero from "./assets/flint-hero-720.jpg";

export interface ReleaseArtwork {
  hero: string;
  alt: string;
  position: string;
}

const DEFAULT_ARTWORK: ReleaseArtwork = {
  hero: defaultHero,
  alt: "Pixel-art Flint logo with a warm ember glow",
  position: "72% center",
};

const VERSION_ARTWORK: Record<string, ReleaseArtwork> = {};

export function artworkForVersion(version?: string): ReleaseArtwork {
  return (version && VERSION_ARTWORK[version]) || DEFAULT_ARTWORK;
}
