FROM nginx:latest

WORKDIR /app

RUN apt-get update 

RUN apt-get install --no-install-recommends -y \
    python3-minimal python3-pip python3-setuptools python3-wheel \
    inotify-tools

COPY requirements.txt /app

RUN pip3 install -r requirements.txt

COPY . /app

CMD ["./scripts/parser.sh"]