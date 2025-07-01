-- Enable PostGIS extension
CREATE EXTENSION IF NOT EXISTS postgis;

-- Create enum for categories
CREATE TYPE business_category AS ENUM (
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

-- Create businesses table
CREATE TABLE businesses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    osm_id BIGINT UNIQUE NOT NULL,
    osm_type VARCHAR(10) NOT NULL, -- 'node', 'way', 'relation'
    name TEXT,
    name_en TEXT,
    name_bg TEXT,
    address TEXT,
    location GEOMETRY(POINT, 4326) NOT NULL,
    email TEXT,
    phone TEXT,
    website TEXT,
    categories business_category[] NOT NULL DEFAULT '{}',
    tags JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    logo_url TEXT,
    logo_map_url TEXT,
    photo_url TEXT,
    is_registered BOOLEAN DEFAULT FALSE
);

-- Create indexes
CREATE INDEX idx_businesses_location ON businesses USING GIST(location);
CREATE INDEX idx_businesses_categories ON businesses USING GIN(categories);
CREATE INDEX idx_businesses_tags ON businesses USING GIN(tags);
CREATE INDEX idx_businesses_name ON businesses (name);
CREATE INDEX idx_businesses_name_en ON businesses (name_en);
CREATE INDEX idx_businesses_name_bg ON businesses (name_bg);
CREATE INDEX idx_businesses_osm_id ON businesses (osm_id);
CREATE INDEX idx_businesses_is_registered ON businesses (is_registered);

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger to automatically update updated_at
CREATE TRIGGER update_businesses_updated_at 
    BEFORE UPDATE ON businesses 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();