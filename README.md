# Shorter

A minimal url shortener witten in rust.
shortener is based on the rocket pastbin example with minimal changes
to provide a filesystem based url shortener with tokio as the async runtime.

This is a minimal functional example without any detailed loggin or security
features.

## usage
```bash
    POST /

        accepts url in the body of the request and responds with the short URL

        EXAMPLE: curl -X POST -d 'https://www.google.com' http://localhost:8000

    GET /<id>

        redirect to long url for `<id>`

        EXAMPLE: curl -I http://localhost:8000/<id>
        
    DELETE /<id>

        deletes the redirect for `<id>`

        EXAMPLE: curl -X DELETE http://localhost:8000/<id>
```
