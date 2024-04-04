-- This file should undo anything in `up.sql`
ALTER TABLE transaction_outs DROP COLUMN runestone;
ALTER TABLE transaction_outs DROP COLUMN cenotaph;
ALTER TABLE transaction_outs DROP COLUMN edicts;
ALTER TABLE transaction_outs DROP COLUMN mint;
ALTER TABLE transaction_outs DROP COLUMN etching;
ALTER TABLE transaction_outs DROP COLUMN burn;
