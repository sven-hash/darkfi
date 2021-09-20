CREATE TABLE IF NOT EXISTS deposit_keypairs(
    d_key_public INTEGER PRIMARY KEY NOT NULL,
   	token_key_private BLOB NOT NULL,
    token_key_public BLOB NOT NULL,
	asset_id BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS withdraw_keypairs(
    token_key_id BLOB PRIMARY KEY NOT NULL,
	d_key_private BLOB NOT NULL,
    d_key_public BLOB NOT NULL,
	asset_id BLOB NOT NULL,
	confirm BLOB NOT NULL
);
