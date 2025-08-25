-- Migration 0015: Revert focus tracking from payments and add focus end timestamp
-- This implements a simpler approach: track when focus ends, compare payment dates

-- Remove focus tracking columns from srp_payments
ALTER TABLE `srp_payments` 
DROP COLUMN `focus_constellation_id`,
DROP COLUMN `focus_constellation_name`;

-- Add focus end timestamp to incursion_focus table
ALTER TABLE `incursion_focus` 
ADD COLUMN `focus_end_timestamp` BIGINT DEFAULT NULL;
