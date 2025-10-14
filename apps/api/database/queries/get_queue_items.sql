SELECT
    `id`,
    `anime_id`,
    `status`,
    `triggered_by_id`
FROM
    `anime_job_queue`
WHERE
    `created_at` < ?;
