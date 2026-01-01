PRAGMA user_version = 2;
DROP VIEW if EXISTS get_collection_posters;

CREATE VIEW get_collection_posters AS SELECT collection_id, poster 
FROM (
	SELECT CASE WHEN NOT movie.fetched THEN NULL ELSE movie.poster END AS poster, item.collection_id
	FROM collection_item item
	JOIN movie ON movie.id = item.media_id
	WHERE item.media_type = 'movie' AND poster IS NOT NULL

	UNION ALL

	SELECT CASE WHEN NOT tv_show.fetched THEN NULL ELSE tv_show.poster END AS poster, item.collection_id
	FROM collection_item item
	JOIN tv_show ON tv_show.id = item.media_id
	WHERE item.media_type = 'show' AND poster IS NOT NULL


	UNION ALL

	SELECT CASE WHEN NOT season.fetched THEN NULL ELSE season.poster END AS poster, item.collection_id
	FROM collection_item item
	JOIN season ON season.id = item.media_id
	WHERE item.media_type = 'season' AND poster IS NOT NULL

	UNION ALL

	SELECT CASE WHEN NOT episode.fetched THEN NULL ELSE episode.poster END AS poster, item.collection_id
	FROM collection_item item
	JOIN episode ON episode.id = item.media_id
	WHERE item.media_type = 'episode' AND episode.poster IS NOT NULL
) 
ORDER BY collection_id
;
