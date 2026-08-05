-- CreateTable
CREATE TABLE `users` (
    `id` VARCHAR(191) NOT NULL,
    `name` VARCHAR(191) NOT NULL,
    `mal_id` INTEGER NOT NULL,
    `mal_access_token` MEDIUMTEXT NOT NULL,
    `mal_refresh_token` MEDIUMTEXT NOT NULL,
    `picture` VARCHAR(191) NOT NULL,
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    `deleted_at` DATETIME(3) NULL,
    `list_last_update` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),

    UNIQUE INDEX `users_name_key`(`name`),
    UNIQUE INDEX `users_mal_id_key`(`mal_id`),
    PRIMARY KEY (`id`)
) DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

-- CreateTable
CREATE TABLE `animes` (
    `id` INTEGER NOT NULL,
    `english_title` VARCHAR(191) NULL,
    `romaji_title` VARCHAR(191) NULL,
    `status` ENUM('FINISHED', 'RELEASING', 'NOT_YET_RELEASED', 'CANCELLED', 'HIATUS') NULL,
    `picture` VARCHAR(191) NULL,
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    `season` VARCHAR(191) NULL,
    `season_year` INTEGER NULL,

    UNIQUE INDEX `animes_id_key`(`id`)
) DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

-- CreateTable
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

-- CreateTable
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

-- CreateTable
CREATE TABLE `anime_relations` (
    `anime_id` INTEGER NOT NULL,
    `relation_id` INTEGER NOT NULL,
    `relation` ENUM('ADAPTATION', 'PREQUEL', 'SEQUEL', 'PARENT', 'SIDE_STORY', 'CHARACTER', 'SUMMARY', 'ALTERNATIVE', 'SPIN_OFF', 'OTHER', 'SOURCE', 'COMPILATION', 'CONTAINS') NOT NULL,

    PRIMARY KEY (`anime_id`, `relation_id`)
) DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

-- CreateTable
CREATE TABLE `anime_series` (
    `series_id` INTEGER NOT NULL,
    `anime_id` INTEGER NOT NULL,
    `series_order` INTEGER NOT NULL,

    PRIMARY KEY (`series_id`, `anime_id`)
) DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

-- CreateTable
CREATE TABLE `anime_users` (
    `user_id` VARCHAR(191) NOT NULL,
    `anime_id` INTEGER NOT NULL,
    `status` ENUM('WATCHING', 'COMPLETED', 'PLAN_TO_WATCH', 'DROPPED', 'ON_HOLD') NOT NULL DEFAULT 'PLAN_TO_WATCH',
    `watch_priority` INTEGER NOT NULL DEFAULT 0,
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),

    INDEX `user_id`(`user_id`),
    INDEX `anime_id`(`anime_id`),
    PRIMARY KEY (`user_id`, `anime_id`)
) DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

-- CreateTable
CREATE TABLE `sessions` (
    `id` VARCHAR(191) NOT NULL,
    `user_id` VARCHAR(191) NOT NULL,
    `expires_at` DATETIME(3) NOT NULL,
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),

    INDEX `user_id`(`user_id`),
    PRIMARY KEY (`id`)
) DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

-- CreateTable
CREATE TABLE `anime_job_queue` (
    `id` VARCHAR(191) NOT NULL,
    `anime_id` INTEGER NOT NULL,
    `status` ENUM('Pending', 'InProgress', 'Failed', 'Complete') NOT NULL,
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    `complete_at` DATETIME(3) NULL,
    `triggered_by_id` VARCHAR(191) NULL,

    PRIMARY KEY (`id`)
) DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

-- AddForeignKey
ALTER TABLE `anime_lists` ADD CONSTRAINT `anime_lists_owner_id_fkey` FOREIGN KEY (`owner_id`) REFERENCES `users`(`id`) ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE `anime_list_entries` ADD CONSTRAINT `anime_list_entries_list_id_fkey` FOREIGN KEY (`list_id`) REFERENCES `anime_lists`(`id`) ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE `anime_list_entries` ADD CONSTRAINT `anime_list_entries_anime_id_fkey` FOREIGN KEY (`anime_id`) REFERENCES `animes`(`id`) ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE `anime_series` ADD CONSTRAINT `anime_series_series_id_fkey` FOREIGN KEY (`series_id`) REFERENCES `animes`(`id`) ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE `anime_series` ADD CONSTRAINT `anime_series_anime_id_fkey` FOREIGN KEY (`anime_id`) REFERENCES `animes`(`id`) ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE `anime_users` ADD CONSTRAINT `anime_users_user_id_fkey` FOREIGN KEY (`user_id`) REFERENCES `users`(`id`) ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE `anime_users` ADD CONSTRAINT `anime_users_anime_id_fkey` FOREIGN KEY (`anime_id`) REFERENCES `animes`(`id`) ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE `sessions` ADD CONSTRAINT `sessions_user_id_fkey` FOREIGN KEY (`user_id`) REFERENCES `users`(`id`) ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE `anime_job_queue` ADD CONSTRAINT `anime_job_queue_triggered_by_id_fkey` FOREIGN KEY (`triggered_by_id`) REFERENCES `users`(`id`) ON DELETE SET NULL ON UPDATE CASCADE;
