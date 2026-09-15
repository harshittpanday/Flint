export interface NewsItem {
  category: string;
  title: string;
  summary: string;
}

export const HOME_NEWS: readonly NewsItem[] = [
  {
    category: "Flint 0.3",
    title: "A stronger launcher foundation",
    summary: "Automatic Java setup, isolated Fabric profiles, safe imports, and local-first launcher integrations.",
  },
  {
    category: "Compatibility",
    title: "Flint Client preview",
    summary: "Local integrations currently support Fabric profiles on Minecraft 1.21.11.",
  },
];
