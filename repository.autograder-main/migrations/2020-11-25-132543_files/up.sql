create table files
(
    id            uuid primary key   default uuid_generate_v4(),
    submission_id uuid      not null,
    updated       timestamp not null default now(),
    encoded_text  bytea     not null,
    constraint fk_submissions
        foreign key (submission_id)
            references submissions (id)
            on delete set null
);
