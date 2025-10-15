SELECT
    count(*) AS importing_count
FROM
    `anime_job_queue`
WHERE
    `triggered_by_id` = ?
    AND `status` IN("Pending", "InProgress")
