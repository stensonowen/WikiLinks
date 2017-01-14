CREATE TABLE paths (
    src     INTEGER NOT NULL,
    dst     INTEGER NOT NULL,
    result  SMALLINT NOT NULL,
    path    INTEGER[] NOT NULL, 
    count   INTEGER NOT NULL DEFAULT 1,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    PRIMARY KEY (src, dst)

);

CREATE TABLE titles (
    title   VARCHAR PRIMARY KEY,
    page_id INTEGER NOT NULL

);

