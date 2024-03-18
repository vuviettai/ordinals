-- Your SQL goes here
CREATE TABLE runes (
  id BIGSERIAL PRIMARY KEY,
  rune VARCHAR NOT NULL,
  tx_height BIGINT NOT NULL,
  rune_index SMALLINT NOT NULL DEFAULT 0
);

CREATE TABLE txid_rune (
  id BIGSERIAL PRIMARY KEY,
  tx_hash VARCHAR NOT NULL,
  rune TEXT NOT NULL
);

CREATE TABLE rune_entries (
  id BIGSERIAL PRIMARY KEY,
  rune_height BIGINT NOT NULL,
  rune_index SMALLINT NOT NULL DEFAULT 0,
  burned BYTEA NOT NULL,
  divisibility SMALLINT NOT NULL,
  etching VARCHAR NOT NULL,
  -- Mint entry
  mint jsonb NULL,
  mints BIGINT NOT NULL,
  rnumber BIGINT NOT NULL,
  spacers INTEGER NOT NULL,
  supply BYTEA NOT NULL,
  symbol CHAR NULL,
  rtimestamp INTEGER NOT NULL
);

CREATE TABLE sequence_number_runeid (
    id BIGSERIAL PRIMARY KEY,
    sequence_number INTEGER NOT NULL,
    tx_hash VARCHAR NOT NULL,
    tx_height BIGINT NOT NULL,
    rune_index SMALLINT NOT NULL DEFAULT 0
);

-- In the ordinals rune balances are stored as a Vec<(u128,u128)>
-- We try store as multiple record with seperated fields: (id: u128; balance: u128)
--
CREATE TABLE outpoint_rune_balances (
    id BIGSERIAL PRIMARY KEY,
    tx_hash VARCHAR NOT NULL,
    vout SMALLINT NOT NULL,
    balance_id VARCHAR NOT NULL,
    balance_value VARCHAR NOT NULL
);

CREATE TABLE block_headers (
    id BIGSERIAL PRIMARY KEY,
    height BIGINT NOT NULL
);

CREATE TABLE outpoint_values (
    id BIGSERIAL PRIMARY KEY,
    tx_hash VARCHAR NOT NULL,
    vout SMALLINT NOT NULL,
    amount VARCHAR NOT NULL
);