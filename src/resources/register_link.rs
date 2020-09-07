use core::fmt;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Mutex;

use actix_web::{Responder, web};
use chrono::prelude::*;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::util::email_sender::DEFAULT_EMAIL_SENDER;
use crate::json_response;

const EXPIRE_TIME: u8 = 30;
const RETRY_TIME: u8 = 60;

lazy_static! {
    static ref EMAIL_CODE_DICT: Mutex<HashMap<String, EmailCodeTimeEntry>> = Mutex::new(HashMap::new());
}

#[derive(Debug)]
pub enum RegisterError {
    NotSUSTech,
    NotStudent,
    CodeInvalid,
    TooMany,
}

impl fmt::Display for RegisterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RegisterError::NotSUSTech => write!(f, "not SUSTech email"),
            RegisterError::NotStudent => write!(f, "not student if you want to register please contact us"),
            RegisterError::TooMany => write!(f, "too many request for link, please wait 60 seconds"),
            RegisterError::CodeInvalid => write!(f, "invalid verification code"),
        }
    }
}

impl Error for RegisterError {}

#[derive(Debug, Clone)]
pub struct EmailCodeTimeEntry {
    entry: EmailCodeEntry,
    last_time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmailCodeEntry {
    email: String,
    pub code: String,
}

pub fn validate_email(email: &str) -> Result<&str, RegisterError> {
    if !email[0..8].parse::<u64>().is_ok() {
        Err(RegisterError::NotStudent)
    } else if !(email[8..].eq("@mail.sustech.edu.cn") || email[8..].eq("@sustech.edu.cn")
        || email[8..].eq("@mail.sustc.edu.cn") || email[8..].eq("@sustc.edu.cn")) {
        Err(RegisterError::NotSUSTech)
    } else {
        Ok(email)
    }
}

pub fn validate_code(email: &str, code: &str) -> Result<(), Box<dyn Error>> {
    let mut email_dict = EMAIL_CODE_DICT.lock()?;
    if let Some(entry) = email_dict.get(email) {
        if Utc::now().signed_duration_since(entry.last_time).num_minutes() < EXPIRE_TIME as i64
            && code == entry.entry.code {
            return Ok(());
        }
        if Utc::now().signed_duration_since(entry.last_time).num_minutes() >= EXPIRE_TIME as i64 {
            email_dict.remove(email);
        }
    }
    Err(Box::new(RegisterError::CodeInvalid))
}

pub async fn get_register_link(email: &str) -> Result<EmailCodeEntry, Box<dyn Error>> {
    let email = validate_email(email)?;

    if let Some(entry) = EMAIL_CODE_DICT.lock()?.get(email) {
        if Utc::now().signed_duration_since(entry.last_time).num_seconds() < RETRY_TIME as i64 {
            return Err(Box::new(RegisterError::TooMany));
        }
    }
    let entry = EmailCodeEntry {
        email: email.to_string(),
        code: Uuid::new_v4().to_string(),
    };
    EMAIL_CODE_DICT.lock()?.insert(email.to_string(), EmailCodeTimeEntry {
        entry: entry.clone(),
        last_time: Utc::now(),
    });
    DEFAULT_EMAIL_SENDER.send(email, "Flow 注册链接", &format!("https://sustechflow.top/signup?vcode={}", entry.code)).await?;
    Ok(entry)
}

pub async fn get_register_link_handler(req: web::Query<&str>) -> impl Responder {
    web::Json(json_response!(get_register_link(req.0).await))
}

