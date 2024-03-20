-- Your SQL goes here
CREATE TABLE statistics (
    id SERIAL PRIMARY KEY,
    schema INTEGER NOT NULL DEFAULT 0,
    blessed_inscriptions INTEGER NOT NULL DEFAULT 0,
    commits INTEGER NOT NULL DEFAULT 0,
    cursed_inscriptions INTEGER NOT NULL DEFAULT 0,
    index_runes BOOLEAN NOT NULL DEFAULT 'true',
    index_sats BOOLEAN NOT NULL DEFAULT 'true',
    lost_sats INTEGER NOT NULL DEFAULT 0,
    outputs_traversed INTEGER NOT NULL DEFAULT 0,
    reserved_runes BIGINT NOT NULL DEFAULT 0,
    runes BIGINT NOT NULL DEFAULT 0,
    satranges BIGINT NOT NULL DEFAULT 0,
    unbound_inscriptions INTEGER NOT NULL DEFAULT 0,
    index_transactions BOOLEAN NOT NULL DEFAULT 'true',
    index_spent_sats BOOLEAN NOT NULL DEFAULT 'true',
    initial_sync_time BIGINT NOT NULL DEFAULT 0
);
-- Height to sequence number
CREATE TABLE height_sequence_numbers (
    id BIGSERIAL PRIMARY KEY,
    height INTEGER NOT NULL DEFAULT 0,
    sequence_number INTEGER NOT NULL DEFAULT 0
)