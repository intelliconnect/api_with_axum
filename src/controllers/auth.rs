use axum::{Extension, Json};
use jsonwebtoken::{encode, Header};
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::{
    error::AppError,
    models::{self, auth::Claims},
    utils::get_timestamp_8_hours_from_now,
    KEYS,
};

pub async fn login(
    Json(credentials): Json<models::auth::User>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Value>, AppError> {
    // check if email or password is a blank string
    if credentials.email.is_empty() || credentials.password.is_empty() {
        return Err(AppError::MissingCredential);
    }

    // get the user for the email from database
    let user = sqlx::query_as::<_, models::auth::User>(
        "SELECT email, password FROM users where email = $1",
    )
    .bind(&credentials.email)
    .fetch_optional(&pool)
    .await
    .map_err(|err| {
        dbg!(err);
        AppError::InternalServerError
    })?;

    if let Some(user) = user {
        //if user exits then:

        // if password is encrypted than decode it first before comparing
        if user.password != credentials.password {
            // password is incorrect
            Err(AppError::WrongCredential)
        } else {
            let claims = Claims {
                email: credentials.email.to_owned(),
                exp: get_timestamp_8_hours_from_now(),
            };
            let token = encode(&Header::default(), &claims, &KEYS.encoding)
                .map_err(|_| AppError::TokenCreation)?;
            // return bearer token
            Ok(Json(json!({ "access_token": token, "type": "Bearer" })))
        }
    } else {
        // if the user does not exit
        Err(AppError::UserDoesNotExist)
    }
}

pub async fn register(
    Json(credentials): Json<models::auth::User>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Value>, AppError> {
    // check if email or password is a blank string
    if credentials.email.is_empty() || credentials.password.is_empty() {
        return Err(AppError::MissingCredential);
    }

    // get the user for the email from database
    let user = sqlx::query_as::<_, models::auth::User>(
        "SELECT email, password FROM users where email = $1",
    )
    .bind(&credentials.email)
    .fetch_optional(&pool)
    .await
    .map_err(|err| {
        dbg!(err);
        AppError::InternalServerError
    })?;

    if let Some(_) = user {
        //if a user with email already exits send error
        return Err(AppError::UserAlreadyExits);
    }

    let result = sqlx::query("INSERT INTO users (email, password) values ($1, $2)")
        .bind(&credentials.email)
        .bind(credentials.password)
        .execute(&pool)
        .await
        .map_err(|_| AppError::InternalServerError)?;
    if result.rows_affected() < 1 {
        Err(AppError::InternalServerError)
    } else {
        Ok(Json(json!({ "msg": "registered successfully" })))
    }
}
