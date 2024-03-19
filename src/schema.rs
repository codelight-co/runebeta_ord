// @generated automatically by Diesel CLI.

diesel::table! {
    block_headers (id) {
        id -> Int8,
        height -> Int8,
        version -> Int4,
        previous_block_hash -> Text,
        merkle_root -> Text,
        time -> Int4,
        bits -> Int4,
        nonce -> Int4,
    }
}

diesel::table! {
    content_type_counts (id) {
        id -> Int4,
        content_type -> Nullable<Text>,
        count -> Int8,
    }
}

diesel::table! {
    height_sequence_numbers (id) {
        id -> Int8,
        height -> Int4,
        sequence_number -> Int4,
    }
}

diesel::table! {
    indexing_block_timestamps (id) {
        id -> Int8,
        block_height -> Int4,
        timestamps -> Int8,
    }
}

diesel::table! {
    inscription_entries (id) {
        id -> Int8,
        charms -> Int2,
        fee -> Int8,
        height -> Int4,
        tx_hash -> Text,
        inscription_index -> Int4,
        inscription_number -> Int4,
        parent -> Nullable<Int4>,
        sat -> Nullable<Int8>,
        sequence_number -> Int4,
        timestamp -> Int4,
    }
}

diesel::table! {
    inscriptions (id) {
        id -> Int8,
        home -> Nullable<Int4>,
        sequence_number -> Int4,
        head -> Text,
        tail -> Text,
        inscription_index -> Int4,
    }
}

diesel::table! {
    outpoint_rune_balances (id) {
        id -> Int8,
        tx_hash -> Varchar,
        vout -> Int4,
        balance_id -> Varchar,
        balance_value -> Varchar,
    }
}

diesel::table! {
    outpoint_satranges (id) {
        id -> Int8,
        tx_hash -> Varchar,
        vout -> Int2,
        range -> Bytea,
    }
}

diesel::table! {
    outpoint_values (id) {
        id -> Int8,
        tx_hash -> Varchar,
        vout -> Int2,
        value -> Int8,
    }
}

diesel::table! {
    rune_entries (id) {
        id -> Int8,
        rune_height -> Int4,
        rune_index -> Int2,
        burned -> Bytea,
        divisibility -> Int2,
        etching -> Varchar,
        mints -> Int8,
        number -> Int8,
        mint -> Nullable<Jsonb>,
        rune -> Text,
        spacers -> Int4,
        supply -> Text,
        #[max_length = 1]
        symbol -> Nullable<Bpchar>,
        timestamp -> Int4,
    }
}

diesel::table! {
    runes (id) {
        id -> Int8,
        rune -> Varchar,
        tx_height -> Int8,
        rune_index -> Int2,
    }
}

diesel::table! {
    satpoints (id) {
        id -> Int8,
        sequence_number -> Int4,
        tx_hash -> Varchar,
        vout -> Int4,
        sat_offset -> Int8,
    }
}

diesel::table! {
    sequence_number_runeids (id) {
        id -> Int8,
        sequence_number -> Int4,
        tx_hash -> Varchar,
        tx_height -> Int8,
        rune_index -> Int2,
    }
}

diesel::table! {
    statistics (id) {
        id -> Int4,
        schema -> Int4,
        blessed_inscriptions -> Int4,
        commits -> Int4,
        cursed_inscriptions -> Int4,
        index_runes -> Bool,
        index_sats -> Bool,
        lost_sats -> Int4,
        outputs_traversed -> Int4,
        reserved_runes -> Int4,
        runes -> Int8,
        satranges -> Int8,
        unbound_inscriptions -> Int4,
        index_transactions -> Bool,
        index_spent_sats -> Bool,
        initial_sync_time -> Int8,
    }
}

diesel::table! {
    transaction_ins (id) {
        id -> Int8,
        tx_hash -> Varchar,
        previous_output_hash -> Varchar,
        previous_output_vout -> Int4,
        script_sig -> Text,
        sequence_number -> Int4,
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
    transactions (id) {
        id -> Int8,
        version -> Int4,
        lock_time -> Int4,
        tx_hash -> Varchar,
    }
}

diesel::table! {
    txid_runes (id) {
        id -> Int8,
        tx_hash -> Varchar,
        rune -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    block_headers,
    content_type_counts,
    height_sequence_numbers,
    indexing_block_timestamps,
    inscription_entries,
    inscriptions,
    outpoint_rune_balances,
    outpoint_satranges,
    outpoint_values,
    rune_entries,
    runes,
    satpoints,
    sequence_number_runeids,
    statistics,
    transaction_ins,
    transaction_outs,
    transactions,
    txid_runes,
);
