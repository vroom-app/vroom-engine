-- Add up migration script here
-- Add new review fields to businesses table
ALTER TABLE search.businesses 
ADD COLUMN average_reviews DECIMAL(3,2) NOT NULL DEFAULT 0.0,
ADD COLUMN review_count INTEGER NOT NULL DEFAULT 0;

-- Update existing rows to have default values
UPDATE search.businesses 
SET average_reviews = 0.0, review_count = 0 
WHERE average_reviews IS NULL OR review_count IS NULL;