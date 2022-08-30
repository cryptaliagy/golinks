set dotenv-load

compose_file := "devstack/docker-compose.yaml"
compose_cmd := "docker compose -f " + compose_file

# Builds the container
build *ARGS:
    {{ compose_cmd }} build

# Runs the container
run *ARGS:
    {{ compose_cmd }} up {{ ARGS }}

down *ARGS:
    {{ compose_cmd }} down {{ ARGS }}


# Builds production container and pushes it to registry
release:
    TARGET=runner {{ compose_cmd }} build --no-cache
    TARGET=runner {{ compose_cmd }} push