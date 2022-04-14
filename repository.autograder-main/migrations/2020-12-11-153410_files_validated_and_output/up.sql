alter table files
    add column validated bool not null default false,
    add column encoded_output bytea;
