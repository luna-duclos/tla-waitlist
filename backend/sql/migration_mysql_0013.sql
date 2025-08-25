-- Migration 0013: Add incursion focus tracking table
-- This table tracks the current highsec incursion focus for SRP voiding

CREATE TABLE `incursion_focus` (
  `id` int NOT NULL AUTO_INCREMENT,
  `current_focus_constellation_id` bigint DEFAULT NULL,
  `current_focus_constellation_name` varchar(255) DEFAULT NULL,
  `last_check_timestamp` bigint NOT NULL,
  `focus_active` tinyint NOT NULL DEFAULT '0',
  `created_at` bigint NOT NULL,
  `updated_at` bigint NOT NULL,
  PRIMARY KEY (`id`),
  CONSTRAINT `incursion_focus_chk_1` CHECK ((`focus_active` in (0,1)))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Add focus_voided_at field to srp_payments table
ALTER TABLE `srp_payments` 
ADD COLUMN `focus_voided_at` bigint DEFAULT NULL;

-- Insert initial record
INSERT INTO `incursion_focus` (`current_focus_constellation_id`, `current_focus_constellation_name`, `last_check_timestamp`, `focus_active`, `created_at`, `updated_at`) 
VALUES (NULL, NULL, UNIX_TIMESTAMP(), 0, UNIX_TIMESTAMP(), UNIX_TIMESTAMP());
