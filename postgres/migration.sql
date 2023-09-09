CREATE TABLE IF NOT EXISTS pessoa
(
    id         uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
    apelido    varchar(32) UNIQUE NOT NULL,
    nome       varchar(100)       NOT NULL,
    nascimento date               NOT NULL,
    stack      text
);
CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE INDEX idx_stack ON pessoa USING gin (stack gin_trgm_ops);

update pg_opclass
set opcdefault = true
where opcname = 'gin_trgm_ops';

CREATE INDEX pessoa_apelido_nome_stack_index
    ON pessoa USING gin (apelido, nome, stack gin_trgm_ops);

CREATE INDEX pessoa_id_index
    ON pessoa USING hash (id);

