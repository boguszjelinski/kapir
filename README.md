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
| /cabs | PUT | Update location of the cab, mark as FREE | Sent: { "Id":2, "Location":123, "Status":"FREE", "Seats": 15}, Received: { "location": 9, "status": "ASSIGNED" }
| /cabs | POST | not used
| /orders | GET | Kabina (customer) can get its orders | a list of orders, see below
| /orders/{id} | GET | inform about a cab assignment |  { "id": 421901,    "status": "COMPLETED", "fromStand": 15, "toStand": 12, "maxWait": 20,    "maxLoss": 1,    "shared": true,    "rcvdTime": "2020-12-22T00:35:54.291618",    "eta": 0,    "inPool": false,    "cab": null,    "customer": {        "id": 5,        "hibernateLazyInitializer": {}    },    "leg": {        "id": 422128,        "fromStand": 15,        "toStand": 14,        "place": 0,        "status": "COMPLETE",        "route": null,        "hibernateLazyInitializer": {}    },    "route": {        "id": 422127,        "status": "COMPLETE",        "cab": {            "id": 165,            "location": 10,            "status": "FREE",            "hibernateLazyInitializer": {}        },        "legs": null,        "hibernateLazyInitializer": {}    }}
| /orders | PUT | accepting, canceling a trip, mark as completed | { "status": "ACCEPTED" }
| /orders | POST | submit a trip request - a cab is needed | {"fromStand": 1, "toStand": 2, "status": "RECEIVED", "maxWait": 10, "maxLoss": 20, "shared": true} 
| /assignfreecab | POST | Customers request a trip in a free cab with Kaut |
| /assigntoroute | POST | Customers enters a cab and tries to join an existing route via Kaut |
| /routes | GET | get ONE route that a cab should follow with all legs | {    "id": 422127,    "status": "ASSIGNED",    "cab": {        "id": 165,        "location": 10,        "status": "FREE",        "hibernateLazyInitializer": {}    },    "legs": [        {            "id": 422128,            "fromStand": 15,            "toStand": 14,            "place": 0,            "status": "COMPLETED",            "route": null        },        {            "id": 422131,            "fromStand": 12,            "toStand": 10,            "place": 3,            "status": "COMPLETE",            "route": null        },        {            "id": 422130,            "fromStand": 13,            "toStand": 12,            "place": 2,            "status": "COMPLETE",            "route": null        },        {            "id": 422129,            "fromStand": 14,            "toStand": 13,            "place": 1,            "status": "COMPLETE",            "route": null        }    ]}
| /routes/{id} | GET | Kabina (customer) gets insight into route and location of the assigned cab | as with /routes
| /routes | PUT | mark as completed  | { "status": "completed" }
| /routewithorders | GET | Kab gets its routes with assigned passengers | as with /routes supplemented by a list of orders assigned to that route
| /legs | PUT | mark as completed  | { "status": "completed" }
| /stops | GET | get all stops | [{"id":0,"bearing":180,"latitude":47.507803,"longitude":19.235276},{"id": ...
| /stops/{id}/traffic | GET | Kavla's source of traffic at the stop | {"stop":{"id":10,"bearing":-179,"latitude":47.492855,"longitude":19.10876,"name":"Ciprus utca"}, "routes":[{"eta":11,"route":{"Id":1043,"Status":"ASSIGNED", "Legs":[{"Id":5747,"RouteId":1043,"From":3575,"To":4846,"Place":0,"Dist":2,"Started":null,"Completed":null,"Status":"ASSIGNED","Passengers":1},{"Id":5995,"RouteId":1043,"From":4846,"To":1468,"Place":1,"Dist":2,"Started":null,"Completed":null,"Status":"ASSIGNED","Passengers":1}], "Cab":{"Id":3575,"Location":3575,"Status":"ASSIGNED","Seats":12}}}], "cabs":[{"Id":5201,"Location":10,"Status":"FREE","Seats":12}]}
| /stats | GET | KPIs, Kanal's source of information | {"kpis":[{"name":"AvgDemandSize","int_val":587},{"name":"AvgExtenderTime",... ], "orders":[{"name":"COMPLETED","int_val":56056},{"name":"PICKEDUP",... ], "cabs":[{"name":"ASSIGNED","int_val":6892},{"name":"FREE",...]}

## Testing
Basic authentication is used, users are identified based on IDs in user name, password is ignored for the time being, nor authorisation is performed (who can call an endpoint).  
curl -H "Content-type: application/json" -X PUT -u cust1:cust1 -d '{ "Id":775791, "Status":"PICKEDUP", "From":0,"To":1,"Wait":10,"Loss":70}' http://localhost:8080/orders

curl -H "Content-type: application/json" -u cab2:cab2 -X PUT -d '{ "Id":2, "Location":123, "Status":"FREE"}' http://localhost:8080/cabs