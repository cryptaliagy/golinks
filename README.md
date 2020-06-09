# Personal GoLinks

Inspired by golinks.io, this uses nginx to run a "personal" golink with redirects. Run this on your local machine and populate the links file with whatever links you want to have handy and it will generate different go/ links for you to use.

## Configuration
1. Add an entry into your `/etc/hosts/` file routing `m` and `go` to `127.0.0.1`
1. Copy the file `data/links.yaml.example` to `data/links.yaml`
1. Add route names and redirect links
1. Save the file

Whenever you want to add more golinks, simply edit the `data/links.yaml` file and save it- nginx will reload automatically!

## Running

> Make sure Docker and Docker-Compose are installed!

This project uses docker and docker-compose.

To run the containers detached from the terminal, use the following command:

```bash
$ make run-detached
```

More `make` commands are available and can be seen in the terminal with `make help` or `make`.


## Usage
Once links have been configured, simply use your browser and point to `go/{url}` with `{url}` being one of the configured route names on the links file. For example, `go/golinks` should redirect to `https://www.golinks.io` and `go/golinks-personal` should redirect to this repository on the default links file.

## Live Demo
Want to see golinks at work without hosting it locally? Try out the live demo, hosted on Google Cloud Platform! 

1. Add the IP `35.208.15.228` to your hosts file for the domain `go/`
1. Add the IP `35.208.15.228` to your hosts file for the domain `m/`
1. Try out some golinks!

Here is the list of golinks and their routes for the live test:

```
go/golinks              ->      golinks.io
go/golinks-personal     ->      github.com/taliamax/golinks
go/natalia              ->      natalia.dev
go/mailme               ->      mailto:iam@natalia.dev
go/gcp                  ->      cloud.google.com
go/aws                  ->      aws.amazon.com
go/azure                ->      azure.microsoft.com
go/pydocs               ->      docs.python.org/3
m/                      ->      mail.google.com
```