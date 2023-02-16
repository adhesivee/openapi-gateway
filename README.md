# OpenAPI Gateway

API Gateway based on OpenAPI routes.

Supports hot reloading of OpenAPI routes.

## Config

```toml
# openapi-gateway.config.toml

# Refresh every minute
reload_cron = "* * * * *"

[[openapi_urls]]
name = "Swagger petstore example#1"
url = "https://petstore3.swagger.io/api/v3/openapi.json"

[[openapi_urls]]
name = "Swagger petstore example#2"
url = "https://petstore3.swagger.io/api/v3/openapi.json"
```

## Start project
```
cargo run
```
Open `http://127.0.0.1:8080/docs/` to find the configured routes.


## Open points
- [ ] OpenAPI YAML file format support
- [ ] Metrics