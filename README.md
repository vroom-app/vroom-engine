# Car Services Backend

A Rust backend service for managing car-related businesses using OpenStreetMap data with PostgreSQL + PostGIS.

## Features

- **OSM Data Import**: Download and parse OpenStreetMap data for car-related businesses
- **PostgreSQL + PostGIS**: Spatial database for storing business locations and data
- **Multi-language Support**: Support for English and Bulgarian business names
- **Comprehensive Business Data**: Stores contact information, categories, and OSM tags
- **Smart Updates**: Only updates unregistered businesses to preserve manual edits

## Quick Start

### Prerequisites

- Rust 1.70+
- PostgreSQL 14+ with PostGIS extension
- Docker Engine

### Setup

1. **Clone and setup the project:**
```bash
git clone https://github.com/vroom-app/vroom-engine
cd vroomgine
```

2. **Create environment file:**
```bash
cp .env.template .env
```

Edit `.env` with your database configuration:
```bash
DATABASE_URL=postgresql://username:password@localhost:5432/car_services
OSM_DATA_URL=https://download.geofabrik.de/europe/bulgaria-latest.osm.pbf
SERVER_PORT=3000
RUST_LOG=info
```

3. **Setup PostgreSQL database:**
```sql
CREATE DATABASE car_services;
CREATE EXTENSION postgis;
```

4. **Run the application:**
```bash
cargo run
```

The server will start on `http://localhost:3000`

### API Endpoints

#### Import OSM Data
```http
POST /api/import/osm
```
Downloads OSM data and imports car-related businesses into the database.

**Response:**
```json
{
  "status": "success",
  "message": "OSM data update completed",
  "stats": {
    "total_parsed": 1250,
    "imported": 1200,
    "failed": 50,
    "success_rate": 96.0
  }
}
```

#### Get Import Status
```http
GET /api/import/status
```
Returns current database statistics.

**Response:**
```json
{
  "total_businesses": 1200,
  "registered_businesses": 45,
  "unregistered_businesses": 1155
}
```

#### Health Check
```http
GET /health
```
Simple health check endpoint.

## Database Schema

### Business Categories

The system supports the following car-related categories:
- `car_wash` - Car wash services
- `car_repair` - Auto repair shops
- `parking` - Parking areas
- `gas_station` - Fuel stations
- `electric_vehicle_charging_station` - EV charging stations
- `car_dealer` - Car dealerships
- `car_rental` - Car rental services
- `detailing_studio` - Car detailing services
- `rims_shop` - Wheel/rim shops
- `tuning` - Car tuning services
- `tire_shop` - Tire shops
- `car_inspection_station` - Vehicle inspection stations

### Business Table Structure

```sql
CREATE TABLE businesses (
    id UUID PRIMARY KEY,
    osm_id BIGINT UNIQUE NOT NULL,
    osm_type VARCHAR(10) NOT NULL,
    name TEXT,
    name_en TEXT,
    name_bg TEXT,
    address TEXT,
    location GEOMETRY(POINT, 4326) NOT NULL,
    email TEXT,
    phone TEXT,
    website TEXT,
    categories business_category[] NOT NULL,
    tags JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    logo_url TEXT,
    logo_map_url TEXT,
    photo_url TEXT,
    is_registered BOOLEAN DEFAULT FALSE
);
```

## OSM Data Processing

The system processes OSM data by:

1. **Downloading**: Fetches OSM PBF files from configured sources
2. **Parsing**: Extracts nodes, ways, and relations with car-related tags
3. **Categorizing**: Maps OSM tags to business categories
4. **Importing**: Upserts data into PostgreSQL with conflict resolution

### Supported OSM Tags

The system recognizes businesses based on these OSM tags:
- `amenity=fuel` → Gas Station
- `amenity=car_wash` → Car Wash
- `amenity=charging_station` → EV Charging Station
- `amenity=parking` → Parking
- `shop=car_repair` → Car Repair
- `shop=car` → Car Dealer
- `shop=tyres` → Tire Shop
- `craft=car_repair` → Car Repair
- And many more...

## Development

### Running Tests
```bash
cargo test
```

### Database Migrations
```bash
# Apply migrations
sqlx migrate run --database-url $DATABASE_URL

# Create new migration
sqlx migrate add <migration_name>
```

### Logging
Set `RUST_LOG` environment variable to control logging level:
```bash
RUST_LOG=debug cargo run
```

## Data Protection

The system protects manually entered data:
- Only businesses with `is_registered=false` are updated during OSM imports
- Registered businesses (`is_registered=true`) are never modified by OSM updates
- This allows for manual curation while keeping OSM data fresh

## Performance Considerations

- Uses spatial indexes for efficient location-based queries
- Bulk operations for OSM data import
- Connection pooling for database access
- Async processing throughout the application

## License

[Your License Here]