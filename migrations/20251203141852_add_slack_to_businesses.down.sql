-- Add down migration script here
ALTER TABLE search.businesses
    DROP COLUMN slack;
