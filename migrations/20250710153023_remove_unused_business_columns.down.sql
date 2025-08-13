-- Add down migration script here
BEGIN;

ALTER TABLE search.businesses
  ADD COLUMN name_bg TEXT,
  ADD COLUMN email TEXT,
  ADD COLUMN phone TEXT,
  ADD COLUMN website TEXT,
  ADD COLUMN tags JSONB DEFAULT '{}'::jsonb,
  ADD COLUMN logo_url TEXT,
  ADD COLUMN photo_url TEXT;

-- Recreate indexes
CREATE INDEX IF NOT EXISTS idx_businesses_name_bg ON search.businesses (name_bg);
CREATE INDEX IF NOT EXISTS idx_businesses_name ON search.businesses (name);
CREATE INDEX IF NOT EXISTS idx_businesses_name_en ON search.businesses (name_en);
CREATE INDEX IF NOT EXISTS idx_businesses_tags ON search.businesses USING GIN (tags);

COMMIT;