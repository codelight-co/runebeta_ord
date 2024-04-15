-- Your SQL goes here
ALTER TABLE public.transaction_outs ALTER COLUMN value TYPE numeric USING value::numeric;
