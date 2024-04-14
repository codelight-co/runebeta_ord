-- This file should undo anything in `up.sql`
ALTER TABLE transaction_rune_entries DROP COLUMN turbo;
DROP INDEX transaction_rune_entries_tx_hash_runeid;