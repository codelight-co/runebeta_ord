-- This file should undo anything in `up.sql`
ALTER TABLE public.transaction_ins ALTER COLUMN previous_output_vout TYPE int4 USING previous_output_vout::int4;
ALTER TABLE public.outpoint_rune_balances ALTER COLUMN vout TYPE int4 USING vout::int4;