-- Your SQL goes here
CREATE TABLE runes (
  id BIGSERIAL PRIMARY KEY,
  rune VARCHAR NOT NULL,
  tx_height BIGINT NOT NULL,
  rune_index SMALLINT NOT NULL DEFAULT 0
);

CREATE TABLE txid_runes (
  id BIGSERIAL PRIMARY KEY,
  tx_hash VARCHAR NOT NULL,
  rune TEXT NOT NULL
);

CREATE TABLE rune_entries (
  id BIGSERIAL PRIMARY KEY,
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
  mint jsonb NULL,
  --U128
  rune TEXT NOT NULL,
  spacers INTEGER NOT NULL,
  --U128
  supply TEXT NOT NULL,
  symbol CHAR NULL,
  timestamp INTEGER NOT NULL
);
-- sequence to runeid
CREATE TABLE sequence_number_runeids (
    id BIGSERIAL PRIMARY KEY,
    sequence_number INTEGER NOT NULL,
    tx_hash VARCHAR NOT NULL,
    tx_height BIGINT NOT NULL,
    rune_index SMALLINT NOT NULL DEFAULT 0
);

-- In the ordinals rune balances are stored as a Vec<(u128,u128)>
-- We try store as multiple record with seperated fields: (id: u128; balance: u128)
--
CREATE TABLE outpoint_rune_balances (
    id BIGSERIAL PRIMARY KEY,
    tx_hash VARCHAR NOT NULL,
    vout INTEGER NOT NULL,
    balance_id VARCHAR NOT NULL,
    balance_value VARCHAR NOT NULL
);

CREATE TABLE block_headers (
    id BIGSERIAL PRIMARY KEY,
    height BIGINT NOT NULL,
    version INTEGER NOT NULL,
    previous_block_hash TEXT NOT NULL, -- BlockHash
    merkle_root TEXT NOT NULL,   -- TxMerkleNode
    time INTEGER NOT NULL,
    bits INTEGER NOT NULL,
    nonce INTEGER NOT NULL
);

CREATE TABLE outpoint_values (
    id BIGSERIAL PRIMARY KEY,
    tx_hash VARCHAR NOT NULL,
    vout SMALLINT NOT NULL,
    value BIGINT NOT NULL
);

CREATE TABLE outpoint_satranges (
    id BIGSERIAL PRIMARY KEY,
    tx_hash VARCHAR NOT NULL,
    vout SMALLINT NOT NULL,
    range BYTEA NOT NULL
);