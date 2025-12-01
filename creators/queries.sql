-- Example DuckDB queries for analyzing scraped creator data
-- Note: EDN files need to be converted to JSON first for DuckDB
-- Consider using `bb` to convert EDN -> JSONL, or query EDN directly from Clojure

-- Platform distribution
-- Shows which link-in-bio platforms creators are using
select platform, count(*) as count
from (
  select unnest(bio_link_analysis).platform as platform
  from read_json('data/scraped/*.json')
)
group by platform
order by count desc;

-- UTM parameter usage
-- Find creators using UTM tracking
select
  instagram_username,
  count(*) filter (where outbound_link.utm is not null) as links_with_utm,
  count(*) as total_links
from (
  select
    instagram_username,
    unnest(unnest(bio_link_analysis).outbound_links) as outbound_link
  from read_json('data/scraped/*.json')
)
group by instagram_username
order by links_with_utm desc;

-- Most common link destinations
-- What are creators linking to?
select
  regexp_extract(outbound_link.url, 'https?://([^/]+)') as domain,
  count(*) as count
from (
  select unnest(unnest(bio_link_analysis).outbound_links) as outbound_link
  from read_json('data/scraped/*.json')
)
group by domain
order by count desc
limit 20;

-- Cookie tracking analysis
-- Which platforms set the most cookies?
select
  platform,
  avg(array_length(cookies)) as avg_cookies,
  max(array_length(cookies)) as max_cookies
from (
  select
    unnest(bio_link_analysis).platform as platform,
    unnest(bio_link_analysis).cookies as cookies
  from read_json('data/scraped/*.json')
)
group by platform
order by avg_cookies desc;

-- HTTP-only cookie usage
-- Security analysis: which platforms use http-only cookies?
select
  platform,
  count(*) filter (where cookie.http_only = true) as http_only_cookies,
  count(*) as total_cookies
from (
  select
    analysis.platform as platform,
    unnest(analysis.cookies) as cookie
  from (
    select unnest(bio_link_analysis) as analysis
    from read_json('data/scraped/*.json')
  )
)
group by platform
order by http_only_cookies desc;
