CREATE TABLE IF NOT EXISTS verifications (
    uid INTEGER PRIMARY KEY NOT NULL,
    user INTEGER NOT NULL,
    name TEXT NOT NULL,
    otp TEXT NOT NULL
);