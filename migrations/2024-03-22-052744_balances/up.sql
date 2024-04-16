-- Your SQL goes here
-- In the ordinals rune balances are stored as a Vec<(u128,u128)>
-- We try store as multiple record with seperated fields: (id: u128; balance: u128)
--
CREATE TABLE outpoint_rune_balances (
    id BIGSERIAL PRIMARY KEY,
    tx_hash VARCHAR NOT NULL,
    vout BIGINT NOT NULL,
    rune_id VARCHAR NOT NULL,
    -- rune_block INTEGER NOT NULL,
    -- rune_tx SMALLINT NOT NULL,
    --For store u128 value
    balance_value NUMERIC(40) NOT NULL
);