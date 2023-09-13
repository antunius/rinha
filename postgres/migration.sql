CREATE TABLE IF NOT EXISTS pessoa
(
    id         uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
    apelido    varchar(32) UNIQUE NOT NULL,
    nome       varchar(100)       NOT NULL,
    nascimento date               NOT NULL,
    stack      text,
    search     TEXT GENERATED ALWAYS AS (
                   LOWER(nome || apelido || coalesce(stack, ''))
                   ) STORED
);
CREATE EXTENSION IF NOT EXISTS pg_trgm;

update pg_opclass
set opcdefault = true
where opcname = 'gin_trgm_ops';

CREATE INDEX tbl_col_gin_trgm_idx ON pessoa USING gin (search gin_trgm_ops);

CREATE INDEX pessoa_id_index
    ON pessoa USING hash (id);

