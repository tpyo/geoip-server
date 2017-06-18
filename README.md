# geoip-server
Fast, lightweight MaxMind GeoIP lookup server written in Rust (experimental)

### Starting the server
Usage: `geoip-server 0.0.0.0:3000 /path/to/GeoIP2-City.mmdb`

### Querying
Make a GET request to `http://localhost:3000/<ip>`

Example response:
```json
{"ip":"169.0.183.91","latitude":-33.9185,"longitude":18.4131,"time_zone":"Africa/Johannesburg","iso_code":"ZA","city":{"de":"Kapstadt","en":"Cape Town","es":"Ciudad del Cabo","fr":"Le Cap","ja":"ケープタウン","pt-BR":"Cidade do Cabo","ru":"Кейптаун"},"subdivisions":[{"en":"Western Cape","pt-BR":"Cabo Ocidental"}],"country":{"de":"Südafrika","en":"South Africa","es":"Sudáfrica","fr":"Afrique du Sud","ja":"南アフリカ","pt-BR":"África do Sul","ru":"ЮАР","zh-CN":"南非"},"registered_country":{"de":"Südafrika","en":"South Africa","es":"Sudáfrica","fr":"Afrique du Sud","ja":"南アフリカ","pt-BR":"África do Sul","ru":"ЮАР","zh-CN":"南非"}}
```

