CREATE TABLE paths (
    id      SERIAL PRIMARY KEY,
    src     INTEGER NOT NULL,
    dst     INTEGER NOT NULL,
    result  SMALLINT NOT NULL,
    path    INTEGER[] NOT NULL,
    count   INTEGER NOT NULL DEFAULT 0
)

