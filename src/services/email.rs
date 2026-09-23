use crate::errors::{AppResult, AppError};
use serde::{Serialize, Deserialize};
use reqwest::{header, Client, StatusCode};

pub struct EmailService<'a> {
    pub to: &'a str,
    pub subject: String,
    pub html: String,
    #[cfg(debug_assertions)]
    pub force_in_debug: bool,
}

impl<'a> EmailService<'a> {
    pub fn new(to: &'a str, subject: String, html: String) -> Self {
        EmailService {
            to,
            subject,
            html,
            #[cfg(debug_assertions)]
            force_in_debug: true
        }
    }

    #[cfg(debug_assertions)]
    pub async fn send(&self) -> AppResult<()> {
        if self.force_in_debug {
            self._send().await
        } else {
            Ok(())
        }
    }

    #[cfg(not(debug_assertions))]
    pub async fn send(&self) -> AppResult<()> {
        self._send().await
    }

    async fn _send(&self) -> AppResult<()> {
        let email = Email {
            sender: Address { 
                email: &std::env::var("EMAIL_DEFAULT_ADDRESS")
                    .expect("EMAIL_DEFAULT_ADDRESS must be set")
            },
            to: vec![Address {
                email: self.to
            }],
            subject: &self.subject,
            html_content: &self.html
        };

        let uri = format!("https://{}/v3/smtp/email", std::env::var("EMAIL_DOMAIN")
            .expect("EMAIL_DOMAIN must be set"));

        let client = Client::new();
        let response = client.post(uri)
            .header(header::ACCEPT, "application/json")
            .header("api-key", std::env::var("EMAIL_KEY")
                .expect("EMAIL_KEY must be set"))
            .json(&email)
            .send().await?;

        if response.status() != StatusCode::CREATED {
            let body = response.bytes().await?;
            match serde_json::from_slice::<EmailServiceResponseError>(&body) {
                Ok(r) => {
                    return Err(AppError::InternalServerError(format!("Error from email service code: {}, message: {}", r.code, r.message)))
                },
                Err(_) => {
                    return Err(AppError::InternalServerError(format!("Unknown error from email service.")))
                }
            }            
        } 

        Ok(())
    }
}

#[derive(Serialize)]
struct Address<'a> {
    pub email: &'a str
}

#[derive(Serialize)]
struct Email<'a> {
    pub sender: Address<'a>,
    pub to: Vec<Address<'a>>,
    pub subject: &'a str,
    #[serde(rename = "htmlContent")]
    pub html_content: &'a str
}

#[derive(Deserialize)]
struct EmailServiceResponseError {
    pub code: String,
    pub message: String
}