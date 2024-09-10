--- clickhouse-client --host localhost --port 19000 --user default --password default --database 7tv
CREATE TABLE IF NOT EXISTS emote_stats (
    emote_id UUID,
    date Date,
    count Int32
)
ENGINE = SummingMergeTree(count)
PARTITION BY sipHash64(emote_id) % 16
ORDER BY (emote_id, date);
