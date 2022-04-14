create table assignments
(
    id             uuid primary key,
    user_id        uuid      not null,
    encoded_input  bytea     not null,
    encoded_output bytea     not null,
    updated        timestamp not null default now()
);

alter table submissions
    add constraint fk_assignments
        foreign key (assignment_id)
            references assignments (id)
            on delete set null;
