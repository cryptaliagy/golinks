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
build: ## Build the docker containers
	docker-compose \
		-f devstack/docker-compose.yaml \
		build

.PHONY: run-detach
run-detach: ## Run the docker containers detached
	docker-compose \
		-f devstack/docker-compose.yaml \
		up -d

.PHONY: run
run: ## Run docker containers attached to the terminal
	docker-compose \
		-f devstack/docker-compose.yaml \
		up

.PHONY: teardown
teardown: ## Shuts down the containers (if running) and removes their images
	docker-compose \
		-f devstack/docker-compose.yaml \
		down --rmi all
