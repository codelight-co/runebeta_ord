-- Your SQL goes here

ALTER TABLE transaction_outs ADD runestone jsonb DEFAULT '{}'::jsonb NOT NULL;
ALTER TABLE transaction_outs ADD cenotaph jsonb DEFAULT '{}'::jsonb NOT NULL;
ALTER TABLE transaction_outs ADD edicts INTEGER DEFAULT 0 NOT NULL;
ALTER TABLE transaction_outs ADD mint BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE transaction_outs ADD etching BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE transaction_outs ADD burn BOOLEAN NOT NULL DEFAULT false;