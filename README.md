# Kapir

Kabina Rest API in Rust

## Build and run
Firstly you need the database schema, consult [Kern](https://gitlab.com/kabina/kern) documentation. Clone Kern's repository and run (adjust the user, password and schema):
```
cd sql
psql -U kabina kabina < create.sql
psql -U kabina -c "COPY stop(id, no, name, latitude, longitude, bearing) FROM 'stops-Budapest-import.csv' DELIMITER ',' CSV HEADER ENCODING 'UTF8';"
```

Make changes in *kapir.toml* (myhost is where API binds to, helps with serving external requests), then run:
```
ulimit -n 100000
cargo build --release
cargo run --release
```
The *ulimit* command helps under heavy load, number has to be adjusted to needs. 

See [readme](https://gitlab.com/kabina/kern/-/blob/master/HOWTORUN.md) how to run all Kabina components in a simulation.

## Endpoints
The following endpoints are available now with described purposes:

| Endpoint | Method | Purpose | Response example
|----------|--------|----------------------------------|-----
| /cabs/{id} | GET | Inform customer about location | { "id": 1, "location": 10, "status": "FREE" }
| /cabs | PUT | Update location of the cab, mark as FREE | { "location": 9, "status": "ASSIGNED" }
| /cabs | POST | not used
| /orders | GET | Kabina (customer) can get its orders | a list of orders
| /orders/{id} | GET | inform about a cab assignment |  { "id": 421901,    "status": "COMPLETED", "fromStand": 15, "toStand": 12, "maxWait": 20,    "maxLoss": 1,    "shared": true,    "rcvdTime": "2020-12-22T00:35:54.291618",    "eta": 0,    "inPool": false,    "cab": null,    "customer": {        "id": 5,        "hibernateLazyInitializer": {}    },    "leg": {        "id": 422128,        "fromStand": 15,        "toStand": 14,        "place": 0,        "status": "COMPLETE",        "route": null,        "hibernateLazyInitializer": {}    },    "route": {        "id": 422127,        "status": "COMPLETE",        "cab": {            "id": 165,            "location": 10,            "status": "FREE",            "hibernateLazyInitializer": {}        },        "legs": null,        "hibernateLazyInitializer": {}    }}
| /orders | PUT | accepting, canceling a trip, mark as completed | { "status": "ACCEPTED" }
| /orders | POST | submit a trip request - a cab is needed | {"fromStand": 1, "toStand": 2, "status": "RECEIVED", "maxWait": 10, "maxLoss": 20, "shared": true} 
| /assignfreecab | POST | Customers request a trip in a free cab with Kaut |
| /assigntoroute | POST | Customers enters a cab and tries to join an existing route via Kaut |
| /routes | GET | get ONE route that a cab should follow with all legs | {    "id": 422127,    "status": "ASSIGNED",    "cab": {        "id": 165,        "location": 10,        "status": "FREE",        "hibernateLazyInitializer": {}    },    "legs": [        {            "id": 422128,            "fromStand": 15,            "toStand": 14,            "place": 0,            "status": "COMPLETED",            "route": null        },        {            "id": 422131,            "fromStand": 12,            "toStand": 10,            "place": 3,            "status": "COMPLETE",            "route": null        },        {            "id": 422130,            "fromStand": 13,            "toStand": 12,            "place": 2,            "status": "COMPLETE",            "route": null        },        {            "id": 422129,            "fromStand": 14,            "toStand": 13,            "place": 1,            "status": "COMPLETE",            "route": null        }    ]}
| /routes/{id} | GET | Kabina (customer) gets insight into route and location of the assigned cab | as with /routes
| /routes | PUT | mark as completed  | { "status": "completed" }
| /routeswithorders | GET | Kab gets its routes with assigned passengers | as with /routes supplemented by a list of orders assigned to that route
| /legs | PUT | mark as completed  | { "status": "completed" }
| /stops | GET | get all stops | [{"id":0,"bearing":180,"latitude":47.507803,"longitude":19.235276},{"id": ...
| /stops/{id}/traffic | GET | Kavla's source of traffic at the stop | It returns the stop, routes going through it and cabs assigned to these routes
| /stats | GET | Kanal's source of information | List of KPIs with values and number of orders and cabs with different statuses