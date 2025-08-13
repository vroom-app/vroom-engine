-- Add up migration script here
ALTER TABLE search.businesses
  DROP COLUMN IF EXISTS specializations,
  ADD COLUMN specializations text[]
    DEFAULT ARRAY[]::text[];