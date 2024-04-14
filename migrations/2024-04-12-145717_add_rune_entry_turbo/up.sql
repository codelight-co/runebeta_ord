-- Your SQL goes here
ALTER TABLE transaction_rune_entries ADD turbo BOOLEAN NOT NULL DEFAULT false;
CREATE UNIQUE INDEX transaction_rune_entries_tx_hash_runeid ON public.transaction_rune_entries USING btree (tx_hash, rune_id);