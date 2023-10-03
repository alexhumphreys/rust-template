DATE := $(shell date "+%y%m%d%H%M")
DATABASE_URL ?= postgres://test-user:123@localhost:5432/test-db
DOCKER_NETWORK := rust-template

db-connect:
	pgcli $(DATABASE_URL)

db-migrate:
	cd api/ && cargo sqlx migrate run --database-url $(DATABASE_URL)

db-run:
	docker network create $(DOCKER_NETWORK) || true
	docker run -it -p 5432:5432 --network $(DOCKER_NETWORK) --name some-postgres -e POSTGRES_PASSWORD=123 -e POSTGRES_USER=test-user -e POSTGRES_DB=test-db -d postgres -c shared_preload_libraries='pg_stat_statements'

db-stop:
	docker stop some-postgres || true

db-start: docker-compose-down
	docker start some-postgres

db-run-pganalyze:
	docker run --rm --network prediction-game --env-file pganalyze_collector.env quay.io/pganalyze/collector:stable collector

db-delete:
	docker rm -f some-postgres

docker-compose-build dcb:
	docker compose build

docker-compose-up dcu: db-stop
	docker compose up

docker-compose-down dcd: docker-compose-down-int db-start

docker-compose-down-int:
	docker compose down

openapi-generate: db-start
	OASGEN_WRITE_SPEC=1 cd api/ && cargo build
	cd ..
	openapi-generator generate -g rust -i ./api/openapi.yaml -o api-client --package-name api-client --artifact-version 0.0.1
