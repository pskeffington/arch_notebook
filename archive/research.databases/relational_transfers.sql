-- Synthetic relational source for the SQL ingestion guide.
-- Run only against a disposable SQLite database; these statements replace
-- the fixture's accounts and transfers tables.
DROP TABLE IF EXISTS transfers;
DROP TABLE IF EXISTS accounts;

CREATE TABLE accounts (
    id INTEGER PRIMARY KEY,
    owner_name TEXT NOT NULL,
    institution TEXT NOT NULL
);

CREATE TABLE transfers (
    id TEXT PRIMARY KEY,
    sender_account_id INTEGER NOT NULL REFERENCES accounts(id),
    recipient_account_id INTEGER NOT NULL REFERENCES accounts(id),
    transfer_type TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    amount REAL NOT NULL,
    status TEXT NOT NULL
);

INSERT INTO accounts (id, owner_name, institution) VALUES
    (101, 'Mara Venn', 'Northstar Credit'),
    (102, 'Elias Ward', 'Harbor Cooperative'),
    (103, 'Jun Park', 'Northstar Credit'),
    (104, 'Oriole Freight', 'Meridian Commercial');

INSERT INTO transfers
    (id, sender_account_id, recipient_account_id, transfer_type, occurred_at, amount, status)
VALUES
    ('T-2025-0104', 101, 102, 'WIRE', '2025-01-04', 840.00, 'posted'),
    ('T-2025-0112', 103, 104, 'ACH', '2025-01-12', 2150.50, 'posted'),
    ('T-2025-0127', 102, 104, 'WIRE', '2025-01-27', 420.00, 'reviewed'),
    ('T-2025-0203', 101, 104, 'ACH', '2025-02-03', 910.00, 'posted'),
    ('T-2025-0211', 104, 103, 'WIRE', '2025-02-11', 1860.00, 'reviewed'),
    ('T-2025-0224', 102, 101, 'CASH', '2025-02-24', 305.25, 'posted');
