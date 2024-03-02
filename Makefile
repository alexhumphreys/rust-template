DATE := $(shell date "+%y%m%d%H%M")
DATABASE_URL ?= postgres://test-user:123@127.0.0.1:5432/test-db
DOCKER_NETWORK := rust-template

db-connect:
	pgcli $(DATABASE_URL)

db-migrate:
	cd api/ && cargo sqlx migrate run --database-url $(DATABASE_URL)

db-add-migration:
	cd api/ && cargo sqlx migrate add

db-migrate-prepare:
	cd api/ && cargo sqlx prepare

db-run:
	docker network create $(DOCKER_NETWORK) || true
	docker run -it -p 5432:5432 --network $(DOCKER_NETWORK) --name some-postgres -e POSTGRES_PASSWORD=123 -e POSTGRES_USER=test-user -e POSTGRES_DB=test-db -d postgres -c shared_preload_libraries='pg_stat_statements'

db-stop:
	docker stop some-postgres || true

db-start:
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

clean:
	cargo clean
	cd api/ && cargo clean
	cd website/ && cargo clean

docker-compose-loco-build:
	docker compose -f ./docker-compose.loco-base.yaml -f ./docker-compose.loco-apps.yaml build

docker-compose-loco-up:
	docker compose -f ./docker-compose.loco-base.yaml -f ./docker-compose.loco-apps.yaml up

docker-compose-loco-down:
	docker compose -f ./docker-compose.loco-base.yaml -f ./docker-compose.loco-apps.yaml down

docker-compose-loco-restart: docker-compose-loco-down docker-compose-loco-build docker-compose-loco-up

openfga-transform:
	fga model transform --input-format fga --file ./openfga/model.fga > ./openfga/model.json
