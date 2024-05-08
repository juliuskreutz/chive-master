CREATE TABLE IF NOT EXISTS warns (
    id integer PRIMARY KEY AUTOINCREMENT NOT NULL,
    user integer NOT NULL,
    moderator integer NOT NULL,
    reason text NOT NULL,
    dm boolean NOT NULL,
    message text
);

