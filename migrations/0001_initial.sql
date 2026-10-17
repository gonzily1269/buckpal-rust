CREATE TABLE account (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    opening_balance INTEGER NOT NULL CHECK (opening_balance >= 0)
);

CREATE TABLE transfer (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_account_id INTEGER NOT NULL REFERENCES account(id),
    target_account_id INTEGER NOT NULL REFERENCES account(id),
    amount INTEGER NOT NULL CHECK (amount > 0),
    created_at TEXT NOT NULL,
    CHECK (source_account_id <> target_account_id)
);

CREATE TABLE activity (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transfer_id INTEGER NOT NULL REFERENCES transfer(id),
    owner_account_id INTEGER NOT NULL REFERENCES account(id),
    source_account_id INTEGER NOT NULL REFERENCES account(id),
    target_account_id INTEGER NOT NULL REFERENCES account(id),
    amount INTEGER NOT NULL CHECK (amount > 0),
    created_at TEXT NOT NULL,
    UNIQUE (transfer_id, owner_account_id)
);

CREATE INDEX activity_owner_id_idx ON activity(owner_account_id, id DESC);

INSERT INTO account (id, name, opening_balance) VALUES
    (1, 'Everyday account', 5000),
    (2, 'Savings account', 2500);
