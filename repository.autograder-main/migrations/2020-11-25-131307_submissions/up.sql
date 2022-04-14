create table submissions
(
    id            uuid primary key   default uuid_generate_v4(),
    assignment_id uuid      not null,
    user_id       uuid      not null,
    extension     text      not null,
    created       timestamp not null default now(),
    update_count  smallint  not null default 0 check ( update_count >= 0 ),
    unique (assignment_id, user_id)
);
