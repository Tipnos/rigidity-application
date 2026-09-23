pub async fn interpret_args(mut args: Vec<String>) -> () {
    match args[0].to_lowercase().as_str() {
        "db" => {
            args.drain(0..1);
            if let Some(e) = db::interpret(args).await.err() {
                println!("{}", e);
            }
        },
        _ => {
            println!("Unknown module: {}", args[0]);
        },
    }
}

mod db {
    pub async fn interpret(args: Vec<String>) -> Result<(), String> {
        let mut cmd = "--";

        for arg in &args {
            if arg.contains("--") {
                cmd = &arg[..];
            } else {
                match cmd {
                    "--insert" => {
                        run_insert(arg).await?;
                    }
                    _ => {
                        return Err(format!("Unknown command {} of db module", arg));
                    }
                }
            }
        }

        Ok(())
    }

    async fn run_insert(arg: &str) -> Result<(), String> {
        match arg {
            "users" => insert_test_users().await,
            _ => Err(format!("Unknown arg {} of command db --insert", arg)),
        }
    }

    async fn insert_test_users() -> Result<(), String> {
        use crate::database::{self, users as user_dao};
        use crate::services::auth;
        use crate::chrono::NaiveDateTime;

        let nb_users = 10;
        let pass_hash = auth::hash_password("spike").map_err(|e| e.to_string())?;
        let pool = database::connect_database().await;
        let email_confirmation_hash = "toto";

        for i in 0..nb_users {
            user_dao::create(
                &format!("{}@spikegames.eu", i),
                &format!("Spike{}", i),
                &format!("{}", i),
                "Spike",
                &format!("{}", i),
                &pass_hash,
                NaiveDateTime::default(),
                email_confirmation_hash,
                &pool,
            ).await.map_err(|e| e.to_string())?;

            user_dao::confirm_email(email_confirmation_hash, &pool)
                .await.map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}
