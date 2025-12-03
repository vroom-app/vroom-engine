-- Add up migration script here
ALTER TABLE search.businesses
    ADD COLUMN slack TEXT NULL;