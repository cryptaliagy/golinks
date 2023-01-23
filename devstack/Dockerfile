FROM golang:latest as build

WORKDIR /app

COPY go.mod go.sum /app/

RUN go mod download

COPY . /app

RUN GOOS=linux GOARCH=amd64 go build -ldflags="-w -s -extldflags=-static" -tags sqlite_omit_load_extension -o /app/app

CMD ["go", "run", ".", "server"]

FROM scratch as runner

COPY --from=build /app/app /app

ENTRYPOINT ["/app", "server"]