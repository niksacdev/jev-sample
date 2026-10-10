CREATE TABLE IF NOT EXISTS comparisons(id INTEGER PRIMARY KEY, snapshot TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS admissions(request_key TEXT PRIMARY KEY, payload_key TEXT NOT NULL, comparison_id INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS events(comparison_id INTEGER NOT NULL, run_id TEXT NOT NULL, sequence INTEGER NOT NULL, event TEXT NOT NULL, PRIMARY KEY(comparison_id,run_id,sequence));
