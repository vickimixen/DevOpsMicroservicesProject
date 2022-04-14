table! {
    assignments (id) {
        id -> Uuid,
        user_id -> Uuid,
        encoded_input -> Bytea,
        encoded_output -> Bytea,
        updated -> Timestamp,
    }
}

table! {
    files (id) {
        id -> Uuid,
        submission_id -> Uuid,
        updated -> Timestamp,
        encoded_text -> Bytea,
        scheduled -> Bool,
        validated -> Bool,
        encoded_output -> Nullable<Bytea>,
    }
}

table! {
    submissions (id) {
        id -> Uuid,
        assignment_id -> Uuid,
        user_id -> Uuid,
        extension -> Text,
        created -> Timestamp,
        update_count -> Int2,
    }
}

joinable!(files -> submissions (submission_id));
joinable!(submissions -> assignments (assignment_id));

allow_tables_to_appear_in_same_query!(assignments, files, submissions,);
