# geoip-server
[![Build](https://github.com/tpyo/geoip-server/actions/workflows/build.yml/badge.svg)](https://github.com/tpyo/geoip-server/actions/workflows/build.yml)

Fast, lightweight MaxMind GeoIP lookup server written in Rust (experimental)

### Starting the server
Usage: `./geoip-server localhost:3000 MaxMind-DB/test-data/GeoIP2-City-Test.mmdb`

### Querying
Make a GET request to http://localhost:3000
```sh
curl http://localhost:3000/89.160.20.128
```

Example response:
```json
{"city":{"geoname_id":2694762,"names":{"de":"Linköping","en":"Linköping","fr":"Linköping","ja":"リンシェーピング","zh-CN":"林雪平"}},"continent":{"code":"EU","geoname_id":6255148,"names":{"de":"Europa","en":"Europe","es":"Europa","fr":"Europe","ja":"ヨーロッパ","pt-BR":"Europa","ru":"Европа","zh-CN":"欧洲"}},"country":{"geoname_id":2661886,"is_in_european_union":true,"iso_code":"SE","names":{"de":"Schweden","en":"Sweden","es":"Suecia","fr":"Suède","ja":"スウェーデン王国","pt-BR":"Suécia","ru":"Швеция","zh-CN":"瑞典"}},"location":{"accuracy_radius":76,"latitude":58.4167,"longitude":15.6167,"metro_code":null,"time_zone":"Europe/Stockholm"},"postal":null,"registered_country":{"geoname_id":2921044,"is_in_european_union":true,"iso_code":"DE","names":{"de":"Deutschland","en":"Germany","es":"Alemania","fr":"Allemagne","ja":"ドイツ連邦共和国","pt-BR":"Alemanha","ru":"Германия","zh-CN":"德国"}},"represented_country":null,"subdivisions":[{"geoname_id":2685867,"iso_code":"E","names":{"en":"Östergötland County","fr":"Comté d'Östergötland"}}],"traits":null}
```

