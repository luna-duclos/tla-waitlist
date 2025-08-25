-- SRP (Ship Replacement Program) functionality
-- Migration 0010: Add missing fields to srp_payment_status table for new SRP logic

-- Add missing fields to srp_payment_status table
ALTER TABLE `srp_payment_status` 
ADD COLUMN `coverage_type` enum('daily','per_focus') NULL AFTER `status`,
ADD COLUMN `coverage_end` bigint NULL AFTER `coverage_type`;

-- Update existing records to have default values
UPDATE `srp_payment_status` SET 
`coverage_type` = 'daily',
`coverage_end` = `payment_date` + 86400 -- Default to 1 day after payment
WHERE `coverage_type` IS NULL;

-- Make the fields NOT NULL after setting default values
ALTER TABLE `srp_payment_status` 
MODIFY COLUMN `coverage_type` enum('daily','per_focus') NOT NULL,
MODIFY COLUMN `coverage_end` bigint NOT NULL;

-- Add index for coverage_end for efficient queries
ALTER TABLE `srp_payment_status` 
ADD INDEX `coverage_end` (`coverage_end`);

-- Add index for coverage_type
ALTER TABLE `srp_payment_status` 
ADD INDEX `coverage_type` (`coverage_type`);
