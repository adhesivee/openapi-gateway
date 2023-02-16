# OpenAPI Gateway

API Gateway that build its routes via [OpenAPI specification](https://swagger.io/specification/).

Hot reloading of OpenAPI files is supported with `reload_cron` (see Config).

This project simplifies orchestration of services that work with OpenAPI.

## Config

```toml
# openapi-gateway-config.toml

# Refresh every minute
reload_cron = "* * * * *"

[[openapi_urls]]
name = "Swagger petstore example V2#JSON"
url = "https://petstore.swagger.io/v2/swagger.json"

[[openapi_urls]]
name = "Swagger petstore example V2#YAML"
url = "https://petstore.swagger.io/v2/swagger.yaml"

[[openapi_urls]]
name = "Swagger petstore example V3#JSON"
url = "https://petstore3.swagger.io/api/v3/openapi.json"

[[openapi_urls]]
name = "Swagger petstore example V3#YAML"
url = "https://petstore3.swagger.io/api/v3/openapi.yaml"
```

## Start project

### Cargo run
```
cargo run
```

### Docker
```
docker-compose -f docker-compose.example.yml up
```

### After project setup

Open `http://127.0.0.1:8080/docs/` to find the configured routes.


## Open points
- [ ] Metrics
- [ ] Tags based inclusion/exclusion