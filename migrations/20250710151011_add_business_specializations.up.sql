-- Add up migration script here
CREATE TYPE search.business_specialization AS ENUM (
  'long_term_rental',
  'non_stop_support',
  'secured_parking',
  'video_surveillance',
  'electric_barrier',
  'ev_charging',
  'comes_to_address',
  'inspection_at_address',
  'pickup_delivery',
  'bmw_software',
  'remote_services',
  'chip_tuning',
  'car_software',
  'extreme_machines',
  'wheel_restoration',
  'nationwide_delivery',
  'europe_import',
  'custom_search',
  'painting_services',
  'body_repair',
  'technical_inspection',
  'dzi',
  'bul_ins',
  'lev_ins',
  'generali',
  'asset',
  'bulstrad'
);

ALTER TABLE search.businesses
  ADD COLUMN specializations search.business_specialization[]
    NOT NULL DEFAULT ARRAY[]::search.business_specialization[];