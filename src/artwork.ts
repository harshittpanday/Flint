import minecraftLandscapeHero from "./assets/minecraft-landscape-hero.webp";

export interface ReleaseArtwork {
  image: string;
  video?: string;
  alt: string;
  position: string;
}

export const DEFAULT_ARTWORK: ReleaseArtwork = {
  image: minecraftLandscapeHero,
  alt: "Cinematic voxel mountain valley at sunrise",
  position: "center center",
};

export const VERSION_ARTWORK: Record<string, ReleaseArtwork> = {};

export function artworkForVersion(version?: string): ReleaseArtwork {
  return (version && VERSION_ARTWORK[version]) || DEFAULT_ARTWORK;
}
