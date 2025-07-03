// @generated automatically by Diesel CLI.

diesel::table! {
    use diesel::sql_types::*;
    
    authentication.token_blacklist (id) {
        id -> Int4,
        #[max_length = 255]
        jti -> Varchar,
        user_id -> Int4,
        expires_at -> Timestamptz,
        blacklisted_at -> Timestamptz,
        #[max_length = 100]
        reason -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    
    authentication.users (id) {
        id -> Int4,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password_hash -> Nullable<Varchar>,
        is_active -> Nullable<Bool>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    token_blacklist,
    users,
); 