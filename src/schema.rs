// @generated automatically by Diesel CLI.

diesel::table! {
    transaction_ins (id) {
        id -> Int8,
        tx_hash -> Varchar,
        previous_output_hash -> Varchar,
        previous_output_vout -> Int4,
        script_sig -> Text,
        sequence_number -> Int8,
        witness -> Text,
    }
}

diesel::table! {
    transaction_outs (id) {
        id -> Int8,
        tx_hash -> Varchar,
        value -> Int8,
        script_pubkey -> Text,
    }
}

diesel::table! {
    transaction_rune_entries (id) {
        id -> Int8,
        tx_hash -> Varchar,
        rune_height -> Int4,
        rune_index -> Int2,
        burned -> Text,
        divisibility -> Int2,
        etching -> Varchar,
        mints -> Int8,
        number -> Int8,
        mint_entry -> Nullable<Jsonb>,
        rune -> Text,
        spacers -> Int4,
        supply -> Text,
        #[max_length = 1]
        symbol -> Nullable<Bpchar>,
        timestamp -> Int4,
    }
}

diesel::table! {
    transactions (id) {
        id -> Int8,
        version -> Int4,
        lock_time -> Int4,
        tx_hash -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    transaction_ins,
    transaction_outs,
    transaction_rune_entries,
    transactions,
);
