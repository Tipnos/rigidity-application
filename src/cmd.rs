use clap::{Subcommand, ValueEnum};

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Database maintenance tasks
    Db {
        #[command(subcommand)]
        action: DbAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum DbAction {
    /// Insert fixture data
    Insert {
        #[arg(value_enum)]
        target: InsertTarget,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum InsertTarget {
    Users,
}

pub async fn run(command: &Command) {
    let result = match command {
        Command::Db { action } => db::run(action).await,
    };

    if let Err(e) = result {
        println!("{}", e);
    }
}

mod db {
    use super::{DbAction, InsertTarget};

    pub async fn run(action: &DbAction) -> Result<(), String> {
        match action {
            DbAction::Insert { target: InsertTarget::Users } => insert_test_users().await,
        }
    }

    // Log in as one of them with `--dev-login` and POST /api-open/dev-login
    async fn insert_test_users() -> Result<(), String> {
        use crate::database::{self, users as user_dao};
        use crate::chrono::NaiveDateTime;

        let nb_users = 10;
        let pool = database::connect_database().await;

        for i in 0..nb_users {
            let user = user_dao::create(
                &format!("Spike{}", i),
                &format!("{}", i),
                "Spike",
                &format!("{}", i),
                NaiveDateTime::default(),
                &pool,
            ).await.map_err(|e| e.to_string())?;

            println!("Created user {} (id {})", user.nickname, user.id);
        }

        Ok(())
    }
}
