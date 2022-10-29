DROP DATABASE IF EXISTS `todo`;
CREATE DATABASE `todo`;
USE `todo`;

CREATE TABLE IF NOT EXISTS `tags` (
  `id` VARBINARY(16) NOT NULL,
  `name` VARCHAR(255) NOT NULL,
  `created_at` DATETIME NOT NULL,
  `updated_at` DATETIME NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `users` (
  `id` VARBINARY(16) NOT NULL,
  `username` VARCHAR(255) NOT NULL UNIQUE,
  `display_name` VARCHAR(255) NOT NULL,
  `hashed_password` VARBINARY(60) NOT NULL,
  `created_at` DATETIME NOT NULL,
  `updated_at` DATETIME NOT NULL,
  `deleted_at` DATETIME,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `todos` (
  `id` VARBINARY(16) NOT NULL,
  `author_id` VARBINARY(16),
  `title` VARCHAR(255) NOT NULL,
  `description` TEXT,
  `created_at` DATETIME NOT NULL,
  `updated_at` DATETIME NOT NULL,

  `state` ENUM('icebox', 'todo', 'in-progress', 'done') NOT NULL DEFAULT 'todo',
  `priority` ENUM('low', 'medium', 'high'),
  `due_date` DATETIME,

  PRIMARY KEY (`id`),
  FOREIGN KEY (`author_id`) REFERENCES `users` (`id`) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `todo_taggings` (
  `todo_id` VARBINARY(16) NOT NULL,
  `tag_id` VARBINARY(16) NOT NULL,
  PRIMARY KEY (`todo_id`, `tag_id`),
  FOREIGN KEY (`todo_id`) REFERENCES `todos` (`id`) ON DELETE CASCADE,
  FOREIGN KEY (`tag_id`) REFERENCES `tags` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;