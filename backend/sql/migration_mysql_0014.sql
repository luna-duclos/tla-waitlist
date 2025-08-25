-- Migration 0014: Add focus tracking to SRP payments
-- This allows tracking which focus each payment was made under

ALTER TABLE `srp_payments` 
ADD COLUMN `focus_constellation_id` BIGINT DEFAULT NULL,
ADD COLUMN `focus_constellation_name` VARCHAR(255) DEFAULT NULL;

-- Update existing payments to have no focus (they'll be voided if there's a current focus)
UPDATE `srp_payments` 
SET `focus_constellation_id` = NULL, `focus_constellation_name` = NULL 
WHERE `focus_constellation_id` IS NULL;
