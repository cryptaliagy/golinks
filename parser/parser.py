import yaml
import os
import sys
import logging
from jinja2 import Template

logging.basicConfig(format="[%(levelname)s] %(message)s", level=logging.INFO)

def main():
    logging.debug('Reading links definition file')
    with open('/app/data/links.yaml') as f:
        links = yaml.load(f, Loader=yaml.Loader)
    
    logging.debug('Reading route template file')
    with open('/app/parser/route_template') as f:
        route_template = Template(f.read())
    
    for link in links['links']:
        redirect = links['links'][link]
        route = link
        logging.debug(f'Data for {link} link')
        logging.debug(f'Redirect url: {redirect}')
        logging.info(f'Rendering link configuration for "{link}"')
        logging.debug(route_template.render(route=route, redirect=redirect))
        (route_template
            .stream(route=route, redirect=redirect)
            .dump(f'/app/nginx/routes/go_{link}.conf'))
    
    for service in links['services']:
        redirect = links['services'][service]
        route = ""
        logging.debug(f'Data for {service} service')
        logging.debug(f'Redirect url: {redirect}')
        logging.info(f'Rendering link configuration for "{service}"')
        logging.debug(route_template.render(route=route, redirect=redirect))
        (route_template
            .stream(route=route, redirect=redirect)
            .dump(f'/app/nginx/routes/{service}.conf'))
    

if __name__ == '__main__':
    main()