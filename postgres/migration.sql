CREATE TABLE IF NOT EXISTS pessoa
(
    id         uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
    apelido    varchar(32) UNIQUE NOT NULL,
    nome       varchar(100)       NOT NULL,
    nascimento date               NOT NULL,
    stack      text
);
CREATE EXTENSION IF NOT EXISTS pg_trgm;

update pg_opclass
set opcdefault = true
where opcname = 'gin_trgm_ops';

CREATE INDEX tbl_col_gin_trgm_idx ON pessoa USING gin (stack gin_trgm_ops, apelido gin_trgm_ops, nome gin_trgm_ops);

CREATE INDEX pessoa_id_index
    ON pessoa USING hash (id);

