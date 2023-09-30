DATE := $(shell date "+%y%m%d%H%M")
DATABASE_URL ?= postgres://test-user:123@localhost:5432/test-db
DOCKER_NETWORK := rust-template

db-connect:
	pgcli $(DATABASE_URL)

db-migrate:
	npx pg-migrations apply --directory migrations --database $(DATABASE_URL)

db-run:
	docker network create $(DOCKER_NETWORK) || true
	docker run -it -p 5432:5432 --network $(DOCKER_NETWORK) --name some-postgres -e POSTGRES_PASSWORD=123 -e POSTGRES_USER=test-user -e POSTGRES_DB=test-db -d postgres -c shared_preload_libraries='pg_stat_statements'

db-run-pganalyze:
	docker run --rm --network prediction-game --env-file pganalyze_collector.env quay.io/pganalyze/collector:stable collector

db-delete:
	docker rm -f some-postgres
