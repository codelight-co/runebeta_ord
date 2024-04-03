// @generated automatically by Diesel CLI.

diesel::table! {
    blocks (id) {
        id -> Int8,
        previous_hash -> Varchar,
        block_hash -> Varchar,
        block_height -> Int8,
        block_time -> Int8,
    }
}

diesel::table! {
    outpoint_rune_balances (id) {
        id -> Int8,
        block_height -> Int8,
        tx_index -> Int4,
        txout_id -> Varchar,
        tx_hash -> Varchar,
        vout -> Int8,
        rune_id -> Varchar,
        address -> Varchar,
        spent -> Bool,
        balance_value -> Numeric,
    }
}

diesel::table! {
    transaction_ins (id) {
        id -> Int8,
        block_height -> Int8,
        tx_index -> Int4,
        tx_hash -> Varchar,
        previous_output_hash -> Varchar,
        previous_output_vout -> Numeric,
        script_sig -> Text,
        script_asm -> Text,
        sequence_number -> Numeric,
        witness -> Text,
    }
}

diesel::table! {
    transaction_outs (id) {
        id -> Int8,
        block_height -> Int8,
        tx_index -> Int4,
        txout_id -> Varchar,
        tx_hash -> Varchar,
        vout -> Numeric,
        value -> Numeric,
        asm -> Varchar,
        dust_value -> Numeric,
        address -> Nullable<Varchar>,
        script_pubkey -> Text,
        spent -> Bool,
        runestone -> Varchar,
        cenotaph -> Varchar,
        edicts -> Int8,
        mint -> Bool,
        etching -> Bool,
        burn -> Bool,
    }
}

diesel::table! {
    transaction_rune_entries (id) {
        id -> Int8,
        block_height -> Int8,
        tx_index -> Int4,
        tx_hash -> Varchar,
        rune_id -> Varchar,
        burned -> Numeric,
        divisibility -> Int2,
        etching -> Varchar,
        parent -> Nullable<Varchar>,
        mints -> Int8,
        number -> Int8,
        mint_entry -> Jsonb,
        rune -> Numeric,
        spacers -> Int4,
        premine -> Int8,
        spaced_rune -> Varchar,
        supply -> Numeric,
        #[max_length = 1]
        symbol -> Nullable<Bpchar>,
        turbo -> Bool,
        timestamp -> Int4,
    }
}

diesel::table! {
    transactions (id) {
        id -> Int8,
        block_height -> Int8,
        tx_index -> Int4,
        version -> Int4,
        lock_time -> Int8,
        tx_hash -> Varchar,
    }
}

diesel::table! {
    txid_rune_addresss (id) {
        id -> Int8,
        block_height -> Int8,
        tx_index -> Int4,
        tx_hash -> Varchar,
        rune_id -> Varchar,
        address -> Varchar,
        spent -> Bool,
    }
}

diesel::table! {
    txid_runes (id) {
        id -> Int8,
        block_height -> Int8,
        tx_index -> Int4,
        tx_hash -> Varchar,
        rune_id -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    blocks,
    outpoint_rune_balances,
    transaction_ins,
    transaction_outs,
    transaction_rune_entries,
    transactions,
    txid_rune_addresss,
    txid_runes,
);
