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
| /cabs/{id} | GET | Inform customer about location | {"Id":7557,"Location":2700,"Status":"FREE","Seats":12}
| /cabs | PUT | Update location of the cab, mark as FREE | Sent: { "Id":2, "Location":123, "Status":"FREE", "Seats": 15}, Received: { "location": 9, "status": "ASSIGNED" }
| /cabs | POST | not used
| /orders | GET | Kabina (customer) can get its orders | a list of orders, see below
| /orders/{id} | GET | inform about a cab assignment | {"Id":21228012,"From":1,"To":2,"Wait":10,"Loss":20,"Distance":12,"Shared":true,"InPool":false,"Status":"RECEIVED","Received":"2025-05-02T11:52:04","Started":null,"Completed":null,"AtTime":null,"Eta":-1,"Cab":{"Id":-1,"Location":-1,"Status":"CHARGING","Seats":-1},"CustId":100100,"RouteId":-1,"LegId":-1}
| /orders | PUT | accepting, canceling a trip, mark as completed | {"Id":21228013, "From": 2, "To": 1, "Status": "PICKEDUP", "Wait": 100, "Loss": 20}
| /orders | POST | submit a trip request - a cab is needed | {"From": 1, "To": 2, "Status": "RECEIVED", "Wait": 10, "Loss": 20, "Shared": true}
| /assignfreecab | POST | Customers request a trip in a free cab with Kaut |
| /assigntoroute | POST | Customers enters a cab and tries to join an existing route via Kaut |
| /routes | GET | get ONE route that a cab should follow with all legs | {"Id":12074,"Status":"ASSIGNED","Legs":[{"Id":27252,"RouteId":12074,"From":659,"To":480,"Place":0,"Dist":1,"Started":"2025-04-29T03:06:07","Completed":"2025-04-29T03:07:07","Status":"COMPLETED","Passengers":0},{"Id":27253,"RouteId":12074,"From":480,"To":2762,"Place":1,"Dist":2,"Started":"2025-04-29T03:08:07","Completed":null,"Status":"STARTED","Passengers":1}],"Cab":{"Id":1579,"Location":480,"Status":"ASSIGNED","Seats":12}}
| /routes/{id} | GET | Kabina (customer) gets insight into route and location of the assigned cab | as with /routes
| /routes | PUT | mark as completed  | {"Id":1, "Status": "COMPLETED"}
| /routewithorders | GET | Kab gets its routes with assigned passengers | as with /routes supplemented by a list of orders assigned to that route
| /legs | PUT | mark as completed  | { "Id":1, "Status": "COMPLETED" }
| /stops | GET | get all stops | [{"id":5191,"bearing":180,"latitude":47.450156,"longitude":19.033194,"name":"Nyírbátor utca"},{"id": ...
| /stops/{id}/traffic | GET | Kavla's source of traffic at the stop | {"stop":{"id":10,"bearing":-179,"latitude":47.492855,"longitude":19.10876,"name":"Ciprus utca"}, "routes":[{"eta":11,"route":{"Id":1043,"Status":"ASSIGNED", "Legs":[{"Id":5747,"RouteId":1043,"From":3575,"To":4846,"Place":0,"Dist":2,"Started":null,"Completed":null,"Status":"ASSIGNED","Passengers":1},{"Id":5995,"RouteId":1043,"From":4846,"To":1468,"Place":1,"Dist":2,"Started":null,"Completed":null,"Status":"ASSIGNED","Passengers":1}], "Cab":{"Id":3575,"Location":3575,"Status":"ASSIGNED","Seats":12}}}], "cabs":[{"Id":5201,"Location":10,"Status":"FREE","Seats":12}]}
| /stats | GET | KPIs, Kanal's source of information | {"kpis":[{"name":"AvgDemandSize","int_val":587},{"name":"AvgExtenderTime",... ], "orders":[{"name":"COMPLETED","int_val":56056},{"name":"PICKEDUP",... ], "cabs":[{"name":"ASSIGNED","int_val":6892},{"name":"FREE",...]}

## Testing
Basic authentication is used, users are identified based on IDs in user name, password is ignored for the time being, nor authorisation is performed (who can call an endpoint). You can send requests manually or via two available client simulators written in Go and Java, which can send thousands requests per minute.

### Curl
curl -H "Content-type: application/json" -X PUT -u cust1:cust1 -d '{ "Id":775791, "Status":"PICKEDUP", "From":0,"To":1,"Wait":10,"Loss":70}' http://localhost:8080/orders

curl -H "Content-type: application/json" -u cab2:cab2 -X PUT -d '{ "Id":2, "Location":123, "Status":"FREE"}' http://localhost:8080/cabs

### Go
'go build' will produce 'kabina' executable. By running './kabina cab' cabs will be simulated, they will send their location and wait for assignement. './kabina' will send custmers' requests. main.go contains simulation parameters, they need to be adjusted. API host address can be set in utils.go. 

### Java
You need to find and download three JAR files (dependencies) to be able to run it, this or newer versions should do - jackson-databind-2.19.0.jar, jackson-core-2.19.0.jar:jackson-annotations-2.19.0.jar. Go to the directory and run these two commands to compile and run (both cabs and customers will be run):

```
javac -d . -cp jackson-databind-2.19.0.jar:jackson-core-2.19.0.jar *.java
java -cp .:jackson-databind-2.19.0.jar:jackson-core-2.19.0.jar:jackson-annotations-2.19.0.jar org.kabina.kat.Main
```
Simulation parameters should be adjusted in Main.java, location of API host can be checked in ApiClient.java.
