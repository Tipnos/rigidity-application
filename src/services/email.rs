use crate::app_conf::config;
use crate::errors::{AppResult, AppError};
use serde::{Serialize, Deserialize};
use reqwest::{header, Client, StatusCode};

pub struct EmailService<'a> {
    pub to: &'a str,
    pub subject: String,
    pub html: String,
}

impl<'a> EmailService<'a> {
    pub fn new(to: &'a str, subject: String, html: String) -> Self {
        EmailService {
            to,
            subject,
            html,
        }
    }

    pub async fn send(&self) -> AppResult<()> {
        let config = config();
        let email = Email {
            sender: Address { 
                email: &config.email_default_address
            },
            to: vec![Address {
                email: self.to
            }],
            subject: &self.subject,
            html_content: &self.html
        };

        let uri = format!("https://{}/v3/smtp/email", config.email_domain);

        let client = Client::new();
        let response = client.post(uri)
            .header(header::ACCEPT, "application/json")
            .header("api-key", &config.email_key)
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