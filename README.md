### WSL rust installation :

for permission purposes run the following command in your UBUNTU bash and not one in your IDE :

- sudo apt update
- sudo apt install build-essential
- sudo apt-get install -y libssl-dev
- sudo apt install pkg-config
- curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

When executing the following command you might encounter the following error *mktemp: too few X's in template ‘rustup’*

It might means that you've got no tmp folder in your root directory or that is read only. 

Optionnal installation : 

- [Cargo watch](https://github.com/passcod/cargo-watch) : cargo install cargo-watch

### Postgres : 

installation :

- sudo apt-get remove postgresql
- sudo apt-get install postgresql

Start service : sudo service postgresql start
Connect to postgres : sudo -u postgres psql

create/alter user :

- sudo -u postgres createuser <username>
- (in postgres prompt) alter user <username> with encrypted password '<password>'

Diesel postgres feature installation : 

- sudo apt install libpq-dev
- cargo install diesel_cli --no-default-features --features postgres
- diesel setup --database-url=postgres://postgres:password@localhost/postgres?sslmode=disable


### Configuration :

The application is configured by CLI arguments or their matching environment variables (a CLI argument takes precedence). Run `cargo run -- --help` to list every option, its env variable and its default.

Defaults target local development. Secrets have no default and must always be provided (see `example.env`):

- cp example.env .env, then fill it in
- set -a && source .env && set +a && cargo run

Production must set, in addition to the secrets:

- DATABASE_URL (e.g. `${POSTGRESQL_ADDON_URI}` on Clever Cloud)
- BASE_URL=https://<domain>
- LISTEN_ADDRESS=0.0.0.0:8080
- MAX_NB_WORKERS, MAX_DB_CONNS_WORKER and RUST_LOG as needed

Maintenance tasks are subcommands, e.g. `cargo run -- db insert users` inserts test users.
