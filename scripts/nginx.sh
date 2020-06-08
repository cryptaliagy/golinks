#!/bin/bash
nginx;
inotifywait -m -r -e modify -e create -e move -e delete /etc/nginx | 
while read events; do
    nginx -s reload;
done;