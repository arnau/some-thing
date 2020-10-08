-- A classifier for a thing.
CREATE TABLE IF NOT EXISTS tag (
    id      text NOT NULL PRIMARY KEY,
    name    text,
    summary text,
    icon    blob
);

-- A collection of things.
CREATE TABLE IF NOT EXISTS collection (
    id      text NOT NULL PRIMARY KEY,
    url     text,
    summary text NOT NULL
);

-- A thing part of a collection.
CREATE TABLE IF NOT EXISTS thing (
    url         text NOT NULL PRIMARY KEY,
    name        text NOT NULL,
    summary     text,
    category_id text NOT NULL,

    FOREIGN KEY (category_id) REFERENCES tag (id)
);

-- A thing belonging to a collection
CREATE TABLE IF NOT EXISTS collection_thing (
    collection_id text NOT NULL,
    thing_id      text NOT NULL,

    PRIMARY KEY (collection_id, thing_id),
    FOREIGN KEY (collection_id) REFERENCES collection (id),
    FOREIGN KEY (thing_id)      REFERENCES thing (id)
);

CREATE TABLE IF NOT EXISTS thing_tag (
    thing_id text NOT NULL,
    tag_id   text NOT NULL,

    PRIMARY KEY (thing_id, tag_id),
    FOREIGN KEY (thing_id) REFERENCES thing (id),
    FOREIGN KEY (tag_id)   REFERENCES tag (id)
);

INSERT OR IGNORE INTO tag VALUES ('miscellaneous', 'Miscellaneous', 'The unclassifiable.', NULL);
