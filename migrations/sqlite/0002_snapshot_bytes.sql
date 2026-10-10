ALTER TABLE comparisons ADD COLUMN snapshot_bytes INTEGER NOT NULL DEFAULT 0;
UPDATE comparisons SET snapshot_bytes = length(CAST(snapshot AS BLOB));
