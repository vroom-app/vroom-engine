-- Add up migration script here
BEGIN;

-- Drop indexes on columns we’re removing
DROP INDEX IF EXISTS idx_businesses_name_bg;
DROP INDEX IF EXISTS idx_businesses_name;
DROP INDEX IF EXISTS idx_businesses_name_en;
DROP INDEX IF EXISTS idx_businesses_tags;

-- Drop the columns
ALTER TABLE search.businesses
  DROP COLUMN IF EXISTS name_bg,
  DROP COLUMN IF EXISTS email,
  DROP COLUMN IF EXISTS phone,
  DROP COLUMN IF EXISTS website,
  DROP COLUMN IF EXISTS tags,
  DROP COLUMN IF EXISTS logo_url,
  DROP COLUMN IF EXISTS photo_url;

COMMIT;