UPDATE
    `anime_job_queue`
SET
    `status` = ?
WHERE
    `status` = ?
ORDER BY
    `created_at` ASC
LIMIT
    ?
