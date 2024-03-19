-- Your SQL goes here
CREATE TABLE transactions (
  id BIGSERIAL PRIMARY KEY,
  version INTEGER NOT NULL,
  lock_time INTEGER NOT NULL,
  tx_hash VARCHAR NOT NULL
);

CREATE TABLE transaction_ins (
  id BIGSERIAL PRIMARY KEY,
  tx_hash VARCHAR NOT NULL,
  previous_output_hash VARCHAR NOT NULL,
  previous_output_vout INTEGER NOT NULL,
  script_sig TEXT NOT NULL,
  sequence_number INTEGER NOT NULL,
  witness TEXT NOT NULL
);

CREATE TABLE transaction_outs (
  id BIGSERIAL PRIMARY KEY,
  tx_hash VARCHAR NOT NULL,
  value BIGINT NOT NULL,
  script_pubkey TEXT NOT NULL
);

CREATE TABLE content_type_counts (
  id SERIAL PRIMARY KEY,
  content_type TEXT NULL,
  count BIGINT NOT NULL
);
-- Home inscriptions
CREATE TABLE inscriptions (
  id BIGSERIAL PRIMARY KEY,
  home INTEGER NULL,
  sequence_number INTEGER NOT NULL,
  head TEXT NOT NULL,
  tail TEXT NOT NULL,
  inscription_index INTEGER NOT NULL
);

CREATE TABLE inscription_entries (
  id BIGSERIAL PRIMARY KEY,
  charms SMALLINT NOT NULL,
  fee BIGINT NOT NULL,
  height INTEGER NOT NULL,
  -- InscriptionId --
  tx_hash TEXT NOT NULL,
  inscription_index INTEGER NOT NULL,
  -- End inscriptionId --
  inscription_number INTEGER NOT NULL,
  parent INTEGER NULL,
  sat BIGINT NULL,
  sequence_number INTEGER NOT NULL,
  timestamp INTEGER NOT NULL
);

CREATE TABLE satpoints (
  id BIGSERIAL PRIMARY KEY,
  sequence_number INTEGER NOT NULL,
  -- OutPoint
  tx_hash VARCHAR NOT NULL,
  vout INTEGER NOT NULL,
  -- Ent Outpoint
  sat_offset BIGINT NOT NULL
);

CREATE TABLE indexing_block_timestamps (
  id BIGSERIAL PRIMARY KEY,
  block_height INTEGER NOT NULL,
  timestamps BIGINT NOT NULL
);