-- Your SQL goes here
CREATE TABLE statistics (
    id SERIAL PRIMARY KEY,
    schema INTEGER NOT NULL DEFAULT 0,
    blessed_inscriptions INTEGER NOT NULL DEFAULT 0,
    commits INTEGER NOT NULL DEFAULT 0,
    cursed_inscriptions INTEGER NOT NULL DEFAULT 0,
    index_runes INTEGER NOT NULL DEFAULT 0,
    index_sats INTEGER NOT NULL DEFAULT 0,
    lost_sats INTEGER NOT NULL DEFAULT 0,
    outputs_traversed INTEGER NOT NULL DEFAULT 0,
    reserved_runes INTEGER NOT NULL DEFAULT 0,
    satranges BIGINT NOT NULL DEFAULT 0,
    unbound_inscriptions INTEGER NOT NULL DEFAULT 0,
    index_transactions INTEGER NOT NULL DEFAULT 0,
    index_spend_sats INTEGER NOT NULL DEFAULT 0,
    initial_sync_time BIGINT NOT NULL DEFAULT 0
);