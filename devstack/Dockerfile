FROM golang:latest as build

WORKDIR /app

COPY . /app

RUN go get .

RUN GOOS=linux CGO_ENABLED=0 go build -o /app/app .

CMD go run . server

FROM debian:10-slim as runner

WORKDIR /app

COPY --from=build /app/app /app/app

CMD ["/app/app", "server", "--port", "80"]