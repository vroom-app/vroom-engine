-- Add down migration script here
-- Drop trigger
DROP TRIGGER IF EXISTS update_businesses_updated_at ON businesses;

-- Drop function
DROP FUNCTION IF EXISTS update_updated_at_column();

-- Drop indexes
DROP INDEX IF EXISTS idx_businesses_location;
DROP INDEX IF EXISTS idx_businesses_categories;
DROP INDEX IF EXISTS idx_businesses_tags;
DROP INDEX IF EXISTS idx_businesses_name;
DROP INDEX IF EXISTS idx_businesses_name_en;
DROP INDEX IF EXISTS idx_businesses_name_bg;
DROP INDEX IF EXISTS idx_businesses_osm_id;
DROP INDEX IF EXISTS idx_businesses_is_registered;

-- Drop table
DROP TABLE IF EXISTS businesses;

-- Drop enum
DROP TYPE IF EXISTS business_category;