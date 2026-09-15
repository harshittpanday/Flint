import type { NewsItem } from "../news";

export function NewsFeed({ items }: { items: readonly NewsItem[] }) {
  return <section className="home-section news-section" aria-labelledby="home-news-title">
    <div className="home-section-heading"><span className="eyebrow">News</span><h2 id="home-news-title">From Flint</h2></div>
    <div className="news-list">
      {items.map((item) => <article className="news-item" key={`${item.category}-${item.title}`}>
        <span>{item.category}</span><strong>{item.title}</strong><p>{item.summary}</p>
      </article>)}
    </div>
  </section>;
}
