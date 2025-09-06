CREATE TABLE IF NOT EXISTS `incursion_focus` (
  `id` int NOT NULL AUTO_INCREMENT,
  `current_focus_constellation_id` bigint DEFAULT NULL,
  `current_focus_constellation_name` varchar(255) DEFAULT NULL,
  `last_check_timestamp` bigint NOT NULL,
  `focus_active` tinyint NOT NULL DEFAULT '0',
  `created_at` bigint NOT NULL,
  `updated_at` bigint NOT NULL,
  `focus_end_timestamp` bigint DEFAULT NULL,
  PRIMARY KEY (`id`),
  CONSTRAINT `incursion_focus_chk_1` CHECK ((`focus_active` in (0,1)))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS `srp_config` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `key` varchar(64) NOT NULL,
  `value` text NOT NULL,
  `etag` varchar(255) DEFAULT NULL,
  `description` text,
  `updated_at` bigint NOT NULL,
  `updated_by_id` bigint NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `key` (`key`),
  KEY `srp_config_ibfk_1` (`updated_by_id`),
  KEY `etag` (`etag`),
  CONSTRAINT `srp_config_ibfk_1` FOREIGN KEY (`updated_by_id`) REFERENCES `character` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS `srp_payments` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `character_name` varchar(255) NOT NULL,
  `payment_amount` decimal(20,2) NOT NULL,
  `payment_date` bigint NOT NULL,
  `expires_at` bigint NOT NULL,
  `coverage_type` enum('daily','per_focus') NOT NULL,
  `created_at` bigint NOT NULL,
  `focus_voided_at` bigint DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `character_name` (`character_name`),
  KEY `expires_at` (`expires_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS `srp_reports` (
  `killmail_link` varchar(512) NOT NULL,
  `submitted_at` bigint NOT NULL,
  `loot_returned` tinyint(1) NOT NULL DEFAULT '0',
  `description` text,
  `submitted_by_id` bigint NOT NULL,
  `status` enum('pending','approved','rejected','paid') NOT NULL DEFAULT 'pending',
  `payout_amount` decimal(20,2) DEFAULT NULL,
  `payout_date` bigint DEFAULT NULL,
  `srp_paid` json DEFAULT NULL,
  `killmail_id` bigint NOT NULL,
  `reason` text,
  `victim_character_name` varchar(255) DEFAULT NULL,
  `victim_ship_type` varchar(255) DEFAULT NULL,
  `fleet_comp` json DEFAULT NULL,
  PRIMARY KEY (`killmail_id`),
  UNIQUE KEY `killmail_link` (`killmail_link`),
  KEY `submitted_by_id` (`submitted_by_id`),
  KEY `status` (`status`),
  KEY `submitted_at` (`submitted_at`),
  CONSTRAINT `srp_reports_submitted_by` FOREIGN KEY (`submitted_by_id`) REFERENCES `character` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS `srp_service_account` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `character_id` bigint NOT NULL,
  `character_name` varchar(255) NOT NULL,
  `corporation_id` bigint NOT NULL,
  `wallet_id` int NOT NULL DEFAULT '1000',
  `access_token` varchar(2048) NOT NULL,
  `refresh_token` varchar(255) NOT NULL,
  `expires` bigint NOT NULL,
  `scopes` varchar(1024) NOT NULL,
  `is_active` tinyint NOT NULL DEFAULT '1',
  `last_used` bigint DEFAULT NULL,
  `created_at` bigint NOT NULL,
  `updated_at` bigint NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `character_id` (`character_id`),
  KEY `corporation_id` (`corporation_id`),
  KEY `is_active` (`is_active`),
  CONSTRAINT `srp_service_account_ibfk_1` FOREIGN KEY (`character_id`) REFERENCES `character` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
