-- Your SQL goes here
ALTER TABLE public.transactions ALTER COLUMN lock_time TYPE int8 USING lock_time::int8;