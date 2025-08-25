-- SRP (Ship Replacement Program) functionality
-- Migration 0011: Simplify SRP tables to basic structure

-- Drop the complex tables
DROP TABLE IF EXISTS `srp_payment_status`;
DROP TABLE IF EXISTS `srp_wallet_journal`;

-- Create simple SRP payments table
CREATE TABLE `srp_payments` (
    `id` bigint NOT NULL AUTO_INCREMENT,
    `character_name` varchar(255) NOT NULL,
    `payment_amount` decimal(20,2) NOT NULL,
    `payment_date` bigint NOT NULL,
    `expire_time` bigint NOT NULL,
    `coverage_type` enum('daily','per_focus') NOT NULL,
    `created_at` bigint NOT NULL,
    PRIMARY KEY (`id`),
    INDEX `character_name` (`character_name`),
    INDEX `expire_time` (`expire_time`)
);

-- Keep the service account table for ESI access
-- Keep the config table for settings
