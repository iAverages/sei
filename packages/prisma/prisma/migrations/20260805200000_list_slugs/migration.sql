ALTER TABLE `anime_lists` ADD COLUMN `slug` VARCHAR(120) NULL;

UPDATE `anime_lists`
SET `slug` = TRIM(BOTH '-' FROM REGEXP_REPLACE(LOWER(`name`), '[^a-z0-9]+', '-'));

UPDATE `anime_lists` SET `slug` = 'list' WHERE `slug` = '';

UPDATE `anime_lists` AS `list`
JOIN (
    SELECT
        `id`,
        ROW_NUMBER() OVER (PARTITION BY `slug` ORDER BY `created_at`, `id`) AS `slug_number`
    FROM `anime_lists`
) AS `numbered` ON `numbered`.`id` = `list`.`id`
SET `list`.`slug` = CONCAT(`list`.`slug`, '-', `numbered`.`slug_number`)
WHERE `numbered`.`slug_number` > 1;

UPDATE `anime_lists` AS `list`
JOIN (
    SELECT `slug`
    FROM `anime_lists`
    GROUP BY `slug`
    HAVING COUNT(*) > 1
) AS `duplicate` ON `duplicate`.`slug` = `list`.`slug`
SET `list`.`slug` = CONCAT(LEFT(`list`.`slug`, 90), '-', `list`.`id`);

UPDATE `anime_lists` AS `list`
JOIN `users` AS `user` ON LOWER(`user`.`name`) = LOWER(`list`.`slug`)
SET `list`.`slug` = CONCAT(LEFT(`list`.`slug`, 90), '-', `list`.`id`);

ALTER TABLE `anime_lists` MODIFY COLUMN `slug` VARCHAR(120) NOT NULL;
CREATE UNIQUE INDEX `anime_lists_slug_key` ON `anime_lists`(`slug`);
