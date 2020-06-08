# Personal GoLinks

Inspired by golinks.io, this uses nginx to run a "personal" golink with redirects. Run this on your local machine and populate the links file with whatever links you want to have handy and it will generate different go/ links for you to use.

## Instructions
1. Copy the file `data/links.yaml.example` to `data/links.yaml`
1. Add route names and redirect links
1. Save the file

Whenever you want to add more golinks, simply edit the `data/links.yaml` file and save it- nginx will reload automatically!