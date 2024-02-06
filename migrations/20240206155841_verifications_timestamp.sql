ALTER TABLE verifications RENAME TO verifications_old;

CREATE TABLE IF NOT EXISTS verifications (
    uid INTEGER PRIMARY KEY NOT NULL,
    user INTEGER NOT NULL,
    name TEXT NOT NULL,
    otp TEXT NOT NULL,
    timestamp TIMESTAMP
);

INSERT INTO verifications(uid, user, name, otp, timestamp) SELECT uid, user, name, otp, CURRENT_TIMESTAMP timestamp FROM verifications_old;
DROP TABLE verifications_old;
