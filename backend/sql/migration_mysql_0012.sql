-- SRP (Ship Replacement Program) functionality
-- Migration 0012: Add etag column to srp_config table for ETag-based caching

-- Add etag column to srp_config table
ALTER TABLE `srp_config` 
ADD COLUMN `etag` varchar(255) NULL AFTER `value`;

-- Add index for etag lookups
ALTER TABLE `srp_config` 
ADD INDEX `etag` (`etag`);
