-- Your SQL goes here
ALTER TABLE public.transaction_ins ALTER COLUMN previous_output_vout TYPE int8 USING previous_output_vout::int8;
ALTER TABLE public.outpoint_rune_balances ALTER COLUMN vout TYPE int8 USING vout::int8;
