table! {
    dialogs (id) {
        id -> Int8,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        computed_id -> Varchar,
        call_id -> Varchar,
        from_tag -> Varchar,
        to_tag -> Varchar,
        flow -> Varchar,
    }
}

table! {
    requests (id) {
        id -> Int8,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        method -> Varchar,
        uri -> Varchar,
        headers -> Text,
        body -> Nullable<Text>,
        raw_message -> Nullable<Text>,
    }
}

table! {
    responses (id) {
        id -> Int8,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        code -> Int2,
        headers -> Text,
        body -> Nullable<Text>,
        raw_message -> Nullable<Text>,
    }
}

table! {
    transactions (id) {
        id -> Int8,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        state -> Varchar,
        branch_id -> Varchar,
        dialog_id -> Int8,
    }
}

joinable!(transactions -> dialogs (dialog_id));

allow_tables_to_appear_in_same_query!(dialogs, requests, responses, transactions,);
