CREATE TABLE `anime_lists` (
    `id` VARCHAR(191) NOT NULL,
    `owner_id` VARCHAR(191) NOT NULL,
    `name` VARCHAR(100) NOT NULL,
    `visibility` ENUM('PRIVATE', 'UNLISTED', 'PUBLIC') NOT NULL DEFAULT 'PRIVATE',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),

    INDEX `anime_lists_owner_id_created_at_id_idx`(`owner_id`, `created_at`, `id`),
    PRIMARY KEY (`id`)
) DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

CREATE TABLE `anime_list_entries` (
    `list_id` VARCHAR(191) NOT NULL,
    `anime_id` INTEGER NOT NULL,
    `position` INTEGER NOT NULL,
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),

    INDEX `anime_list_entries_list_id_position_idx`(`list_id`, `position`),
    INDEX `anime_list_entries_anime_id_idx`(`anime_id`),
    PRIMARY KEY (`list_id`, `anime_id`)
) DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

ALTER TABLE `anime_lists`
    ADD CONSTRAINT `anime_lists_owner_id_fkey`
    FOREIGN KEY (`owner_id`) REFERENCES `users`(`id`)
    ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE `anime_list_entries`
    ADD CONSTRAINT `anime_list_entries_list_id_fkey`
    FOREIGN KEY (`list_id`) REFERENCES `anime_lists`(`id`)
    ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE `anime_list_entries`
    ADD CONSTRAINT `anime_list_entries_anime_id_fkey`
    FOREIGN KEY (`anime_id`) REFERENCES `animes`(`id`)
    ON DELETE RESTRICT ON UPDATE CASCADE;
