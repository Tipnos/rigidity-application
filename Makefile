#!/bin/bash -l
#include .env
SHELL := /bin/bash -l
#-include ./.env
#export $(shell sed 's/=.*//' ./.env)

help: 
	@fgrep -h "##" $(MAKEFILE_LIST) | fgrep -v fgrep | sed -e 's/\\$$//' | sed -e 's/##//'

all: help

migrations_new: ## Create a new migrations
	diesel migration generate $(filter-out $@,$(MAKECMDGOALS))

migrations_run: ## Persist migrations in database
	diesel migration run

migrations_revert: ## Revert the last migration
	diesel migration revert

migrations_redo: ## Redo all migrations
	diesel migration redo

local_start: ## Start everything for local dev
	sudo service postgresql start && set -a && source .env && set +a && cargo run

prod_deploy: ## Run migrations for prod env
	cargo install diesel_cli --no-default-features --features postgres && diesel setup --database-url=${POSTGRESQL_ADDON_URI} && diesel migration run --database-url=${POSTGRESQL_ADDON_URI}
	
