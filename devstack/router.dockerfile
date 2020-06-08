FROM nginx:latest

WORKDIR /code

RUN apt-get update

RUN apt-get install --no-install-recommends -y \
    inotify-tools

COPY . /code

CMD ["./scripts/nginx.sh"]