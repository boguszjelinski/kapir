# Kapir

Kabina Rest API in Rust

## Compile and run
Make changes in *kapir.toml* (myhost is where API binds to, helps with serving external requests), then run:
```
ulimit -n 100000
cargo build --release
cargo run --release
```
The *ulimit* command helps under heavy load, number has to be adjusted to needs. 

## Endpoints
The following endpoints are available now with described purposes:

| Endpoint | Method | Purpose | Response example
|----------|--------|----------------------------------|-----
| /cabs/{id} | GET | Inform customer about location | { "id": 1, "location": 10, "status": "FREE" }
| /cabs | PUT | Update location of the cab, mark as FREE | { "location": 9, "status": "ASSIGNED" }
| /cabs | POST | not used
| /orders/{id} | GET | inform about a cab assignment |  { "id": 421901,    "status": "COMPLETED", "fromStand": 15, "toStand": 12, "maxWait": 20,    "maxLoss": 1,    "shared": true,    "rcvdTime": "2020-12-22T00:35:54.291618",    "eta": 0,    "inPool": false,    "cab": null,    "customer": {        "id": 5,        "hibernateLazyInitializer": {}    },    "leg": {        "id": 422128,        "fromStand": 15,        "toStand": 14,        "place": 0,        "status": "COMPLETE",        "route": null,        "hibernateLazyInitializer": {}    },    "route": {        "id": 422127,        "status": "COMPLETE",        "cab": {            "id": 165,            "location": 10,            "status": "FREE",            "hibernateLazyInitializer": {}        },        "legs": null,        "hibernateLazyInitializer": {}    }}
| /orders | PUT | accepting, canceling a trip, mark as completed | { "status": "ACCEPTED" }
| /orders | POST | submit a trip request - a cab is needed | {"fromStand": 1, "toStand": 2, "status": "RECEIVED", "maxWait": 10, "maxLoss": 20, "shared": true} 
| /routes | GET | get ONE route that a cab should follow with all legs | {    "id": 422127,    "status": "ASSIGNED",    "cab": {        "id": 165,        "location": 10,        "status": "FREE",        "hibernateLazyInitializer": {}    },    "legs": [        {            "id": 422128,            "fromStand": 15,            "toStand": 14,            "place": 0,            "status": "COMPLETED",            "route": null        },        {            "id": 422131,            "fromStand": 12,            "toStand": 10,            "place": 3,            "status": "COMPLETE",            "route": null        },        {            "id": 422130,            "fromStand": 13,            "toStand": 12,            "place": 2,            "status": "COMPLETE",            "route": null        },        {            "id": 422129,            "fromStand": 14,            "toStand": 13,            "place": 1,            "status": "COMPLETE",            "route": null        }    ]}
| /routes | PUT | mark as completed  | { "status": "completed" }
| /legs | PUT | mark as completed  | { "status": "completed" }