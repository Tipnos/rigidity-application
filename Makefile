#!/bin/bash -l
#include .env
SHELL := /bin/bash -l
#-include ./.env
#export $(shell sed 's/=.*//' ./.env)

help: 
	@fgrep -h "##" $(MAKEFILE_LIST) | fgrep -v fgrep | sed -e 's/\\$$//' | sed -e 's/##//'

all: help

migrations_new: ## Create a new migration (make migrations_new name=<name>)
	sqlx migrate add $(name)

migrations_run: ## Persist migrations in database
	sqlx migrate run

local_start: ## Start everything for local dev
	sudo service postgresql start && set -a && source .env && set +a && cargo run

prod_deploy: ## Run migrations for prod env
	cargo install sqlx-cli --no-default-features --features postgres,native-tls && sqlx migrate run --database-url=${POSTGRESQL_ADDON_URI}
	
