.PHONY: help
help: ## Print this help message and exit
	@echo Usage:
	@echo "  make [target]"
	@echo
	@echo Targets:
	@awk -F ':|##' \
		'/^[^\t].+?:.*?##/ {\
			printf "  %-30s %s\n", $$1, $$NF \
		 }' $(MAKEFILE_LIST)

.PHONY: build
build:
	docker-compose \
		-f devstack/docker-compose.yml \
		build

.PHONY: run-detach
run-detach:
	docker-compose \
		-f devstack/docker-compose.yml \
		up -d

.PHONY: run
run:
	docker-compose \
		-f devstack/docker-compose.yml \
		up

.PHONY: teardown
teardown:
	docker-compose \
		-f devstack/docker-compose.yml \
		down --rmi all

.PHONY: rebuild
rebuild:
	make teardown && make build
