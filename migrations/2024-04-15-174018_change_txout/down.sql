-- This file should undo anything in `up.sql`
ALTER TABLE public.transaction_outs ALTER COLUMN value TYPE BIGINT USING value::BIGINT;
ALTER TABLE transaction_outs DROP COLUMN txout_id;