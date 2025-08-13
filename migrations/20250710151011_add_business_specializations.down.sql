-- Add down migration script here
ALTER TABLE search.businesses
  DROP COLUMN IF EXISTS specializations;

DROP TYPE IF EXISTS search.business_specialization;