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