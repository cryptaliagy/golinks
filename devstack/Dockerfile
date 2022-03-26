FROM golang:latest

WORKDIR /app

COPY . /app

RUN go get .

RUN go build -o /app/app .

CMD ["/app/app"]
