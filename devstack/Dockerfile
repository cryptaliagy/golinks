FROM golang:latest as build

WORKDIR /app

COPY go.mod go.sum /app/

RUN go mod download

COPY . /app

RUN GOOS=linux go build -o /app/app .

CMD ["go", "run", ".", "server"]

FROM debian:10-slim as runner

WORKDIR /app

COPY --from=build /app/app /app/app

CMD ["/app/app", "server", "--debug"]