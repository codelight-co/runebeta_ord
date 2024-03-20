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
  -- witness_content TEXT NOT NULL,
  -- witness_elements BIGINT NOT NULL,
  -- witness_indices_start BIGINT NOT NULL
  witness TEXT NOT NULL
);

CREATE TABLE transaction_outs (
  id BIGSERIAL PRIMARY KEY,
  tx_hash VARCHAR NOT NULL,
  value BIGINT NOT NULL,
  script_pubkey TEXT NOT NULL
);

CREATE TABLE transaction_rune_entries (
  id BIGSERIAL PRIMARY KEY,
  tx_hash VARCHAR NOT NULL,
  --RuneId
  rune_height INTEGER NOT NULL,
  rune_index SMALLINT NOT NULL DEFAULT 0,
  --End RuneId
  burned TEXT NOT NULL,
  divisibility SMALLINT NOT NULL,
  etching VARCHAR NOT NULL,
  mints BIGINT NOT NULL,
  number BIGINT NOT NULL,
  -- Mint entry
  mint_entry jsonb NULL,
  --U128
  rune TEXT NOT NULL,
  spacers INTEGER NOT NULL,
  --U128
  supply TEXT NOT NULL,
  symbol CHAR NULL,
  timestamp INTEGER NOT NULL
);