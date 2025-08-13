-- Create search schema
CREATE SCHEMA IF NOT EXISTS search;

-- Enable required extensions in search schema
CREATE EXTENSION IF NOT EXISTS postgis SCHEMA search;
CREATE EXTENSION IF NOT EXISTS pgcrypto SCHEMA search;

-- Create enum for categories in search schema
CREATE TYPE search.business_category AS ENUM (
    'car_wash',
    'car_repair',
    'parking',
    'gas_station',
    'electric_vehicle_charging_station',
    'car_dealer',
    'car_rental',
    'detailing_studio',
    'rims_shop',
    'tuning',
    'tire_shop',
    'car_inspection_station'
);

-- Create businesses table in search schema
CREATE TABLE search.businesses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    osm_id BIGINT,
    name TEXT,
    name_en TEXT,
    name_bg TEXT,
    address TEXT,
    location GEOMETRY(POINT, 4326) NOT NULL,
    email TEXT,
    phone TEXT,
    website TEXT,
    categories search.business_category[] NOT NULL DEFAULT '{}',
    tags JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    logo_url TEXT,
    logo_map_url TEXT,
    photo_url TEXT,
    is_registered BOOLEAN DEFAULT FALSE,
    city TEXT,
    CONSTRAINT unique_osm_id_type UNIQUE (osm_id)
);

-- Indexes on search.businesses
CREATE INDEX idx_businesses_location ON search.businesses USING GIST(location);
CREATE INDEX idx_businesses_categories ON search.businesses USING GIN(categories);
CREATE INDEX idx_businesses_tags ON search.businesses USING GIN(tags);
CREATE INDEX idx_businesses_name ON search.businesses (name);
CREATE INDEX idx_businesses_name_en ON search.businesses (name_en);
CREATE INDEX idx_businesses_name_bg ON search.businesses (name_bg);
CREATE INDEX idx_businesses_is_registered ON search.businesses (is_registered);

-- Function to update updated_at timestamp in search schema
CREATE OR REPLACE FUNCTION search.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Trigger to update updated_at on row updates
CREATE TRIGGER update_businesses_updated_at 
    BEFORE UPDATE ON search.businesses 
    FOR EACH ROW 
    EXECUTE FUNCTION search.update_updated_at_column();