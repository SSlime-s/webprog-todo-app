DROP DATABASE IF EXISTS `todo`;
CREATE DATABASE `todo`;
USE `todo`;

CREATE TABLE IF NOT EXISTS `tags` (
  `id` VARBINARY(16) NOT NULL,
  `name` VARCHAR(255) NOT NULL,
  `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `users` (
  `id` VARBINARY(16) NOT NULL,
  `username` VARCHAR(255) UNIQUE,
  `display_name` VARCHAR(255) NOT NULL,
  `hashed_password` VARBINARY(60) NOT NULL,
  `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `deleted_at` DATETIME,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `state_mapping` (
  `state_id` TINYINT NOT NULL,
  `state_name` varchar(255) NOT NULL UNIQUE,
  PRIMARY KEY (`state_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
INSERT INTO `state_mapping` (`state_id`, `state_name`) VALUES
  (0, 'icebox'),
  (1, 'todo'),
  (2, 'in-progress'),
  (3, 'done')
ON DUPLICATE KEY UPDATE state_id=state_id;

CREATE TABLE IF NOT EXISTS `priority_mapping` (
  `priority_id` TINYINT NOT NULL,
  `priority_name` varchar(255) NOT NULL UNIQUE,
  PRIMARY KEY (`priority_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
INSERT INTO `priority_mapping` (`priority_id`, `priority_name`) VALUES
  (0, 'low'),
  (1, 'medium'),
  (2, 'high')
ON DUPLICATE KEY UPDATE priority_id=priority_id;

CREATE TABLE IF NOT EXISTS `todos` (
  `id` VARBINARY(16) NOT NULL,
  `author_id` VARBINARY(16),
  `title` VARCHAR(255) NOT NULL,
  `description` TEXT,
  `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

  `state` VARCHAR(255) NOT NULL DEFAULT 'todo',
  `priority` VARCHAR(255),
  `due_date` DATETIME,

  PRIMARY KEY (`id`),
  FOREIGN KEY (`author_id`) REFERENCES `users` (`id`) ON DELETE SET NULL,

  FOREIGN KEY (`state`) REFERENCES `state_mapping` (`state_name`) ON UPDATE CASCADE ON DELETE RESTRICT,
  FOREIGN KEY (`priority`) REFERENCES `priority_mapping` (`priority_name`) ON UPDATE CASCADE ON DELETE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `todo_taggings` (
  `todo_id` VARBINARY(16) NOT NULL,
  `tag_id` VARBINARY(16) NOT NULL,
  PRIMARY KEY (`todo_id`, `tag_id`),
  FOREIGN KEY (`todo_id`) REFERENCES `todos` (`id`) ON DELETE CASCADE,
  FOREIGN KEY (`tag_id`) REFERENCES `tags` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
