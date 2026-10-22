CREATE TABLE facts (e TEXT NOT NULL, a TEXT NOT NULL, v TEXT NOT NULL, valid_from INTEGER NOT NULL, valid_to INTEGER);
CREATE INDEX eavt ON facts (e, a, v, valid_from, valid_to);
CREATE INDEX aevt ON facts (a, e, v, valid_from, valid_to);
CREATE INDEX avet ON facts (a, v, e, valid_from, valid_to);
CREATE INDEX vaet ON facts (v, a, e, valid_from, valid_to);
CREATE TABLE txs (tx INTEGER PRIMARY KEY, commit_id TEXT NOT NULL);
