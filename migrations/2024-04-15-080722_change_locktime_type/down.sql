-- This file should undo anything in `up.sql`
ALTER TABLE public.transactions ALTER COLUMN lock_time TYPE int4 USING lock_time::int4;