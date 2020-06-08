#!/bin/bash
mkdir -p ./nginx/sites-enabled ./nginx/routes
python3 ./parser/parser.py;
cp ./nginx/sites-available/* ./nginx/sites-enabled;
inotifywait -r -e modify -m /app/data |
while read events; do
    rm -r ./nginx/routes/*;
    python3 ./parser/parser.py;
done;