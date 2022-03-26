FROM golang:latest

WORKDIR /app

COPY . /app

RUN go get .

# RUN go build -o /app/app .

CMD go build -o /app/bin/ .
