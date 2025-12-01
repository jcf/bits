-- Creator Platform Analysis
-- Run with: duckdb :memory: < analysis.sql

-- Load all JSON files into a table
CREATE TABLE profiles AS
SELECT * FROM read_json('data/json/*.json',
  format='auto',
  union_by_name=true
);

-- Overview: How many profiles and bio links
SELECT
  COUNT(DISTINCT "instagram/username") as total_profiles,
  COUNT(*) as total_profiles_with_data,
  SUM(len("instagram/bio-links")) as total_bio_links
FROM profiles;

-- Platform breakdown: Which link-in-bio platforms are used
WITH bio_links AS (
  SELECT
    "instagram/username" as username,
    UNNEST("bio-link-analysis") as link_data
  FROM profiles
)
SELECT
  link_data.platform as platform,
  COUNT(*) as usage_count,
  COUNT(DISTINCT username) as unique_creators,
  ROUND(COUNT(*) * 100.0 / SUM(COUNT(*)) OVER (), 2) as percentage
FROM bio_links
WHERE link_data.platform IS NOT NULL
GROUP BY link_data.platform
ORDER BY usage_count DESC;

-- Destination pages captured
WITH bio_links AS (
  SELECT
    "instagram/username" as username,
    UNNEST("bio-link-analysis") as link_data
  FROM profiles
)
SELECT
  link_data.platform as platform,
  link_data.url as destination_url,
  link_data.screenshot as screenshot_path,
  username
FROM bio_links
ORDER BY platform, username;


-- Screenshot inventory
WITH bio_links AS (
  SELECT
    "instagram/username" as username,
    UNNEST("bio-link-analysis") as link_data
  FROM profiles
)
SELECT
  COUNT(*) as total_destination_screenshots,
  COUNT(DISTINCT username) as profiles_with_destinations,
  list(DISTINCT link_data.platform ORDER BY link_data.platform) as platforms_captured
FROM bio_links
WHERE link_data.screenshot IS NOT NULL;
