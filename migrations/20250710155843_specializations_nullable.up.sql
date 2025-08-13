-- Add up migration script here
ALTER TABLE search.businesses
  DROP COLUMN IF EXISTS specializations,
  ADD COLUMN specializations search.business_specialization[]
    DEFAULT ARRAY[]::search.business_specialization[];