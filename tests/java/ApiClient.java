/*
 * Copyright 2025 Bogusz Jelinski
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
package org.kabina.kat;

import java.net.HttpURLConnection;
import java.util.ArrayList;
import java.util.Base64;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.net.URL;
import java.io.OutputStream;
import java.io.BufferedReader;
import java.io.FileInputStream;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.time.LocalDateTime;
import java.time.format.DateTimeFormatter;
import java.io.IOException;
import java.util.logging.Logger;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;

public class ApiClient {

    protected ObjectMapper objectMapper;
    protected Logger logger;

    private static final String HOST = "http://localhost:8080/";
    private static final String ROUTES = "routes";

    public ApiClient() {
        this.objectMapper = new ObjectMapper();
    }

    protected Demand saveOrder(String method, Demand d, int usrId) {
        String json = "{\"From\":" + d.from + ", \"To\": " + d.to + ", \"Status\":\"" + d.status
                            + "\", \"Wait\":" + d.maxWait + ", \"Loss\": "+ d.maxLoss
                            + ", \"Shared\": true";
        if (d.atTime != null) {
            json += ", \"AtTime\": \"" + date2String(d.atTime) + "\"";
        }
        if (method.equals("PUT")) {
            json += ", \"Id\": " + d.id;
        }
        json += "}"; // TODO: hardcode, should be a parameter too
        json = saveJSON(method, "orders", "cust" + usrId, d.id, json); // TODO: FAIL, this is not customer id
        if (json == null || json.length() == 0) {
            logger.info("Method " + method + " on 'orders' rejected for cust_id=" + usrId + ", order_id=" + d.id);
            return null;
        }
        return getOrderFromJson(json);
    }

    private String date2String (LocalDateTime tm) {
        //  DateTimeFormatter.ISO_DATE_TIME
        DateTimeFormatter formatter = DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm:ss");
        return tm.format(formatter);
    }

    protected Demand getOrder(int userId, int orderId) {
        String json = getEntityAsJson("cust" + userId, "orders/" + orderId);
        return getOrderFromJson(json);
    }

    protected Cab getCab(String entityUrl, int userId, int id) {
        String json = getEntityAsJson("cust" + userId, entityUrl + id);
        return getCabFromJson(json);
    }

    protected Cab getCabAsCab(String entityUrl, int userId, int id) {
        String json = getEntityAsJson("cab"+userId, entityUrl + id);
        return getCabFromJson(json);
    }

    protected List<Stop> getStops(String entityUrl, int userId) {
        String json = getEntityAsJson("cab" + userId, entityUrl);
        return getStopsFromJson(json);
    }

    protected void updateCab(int cabId, Cab cab) {
        String json = "{\"Location\":" + cab.location + ", \"Status\": \""+ cab.status +"\"," +
                        "\"Id\":" + cabId +"}";
        log("cab", cabId, json);
        saveJSON("PUT", "cabs", "cab" + cabId, cabId, json);
    }

    protected void updateRoute(int cabId, Route r) {
        String json = "{\"Id\":" + r.id + ", \"Status\":\"" + r.status +"\"}";
        log("route", r.id, json);
        saveJSON("PUT", ROUTES, "cab" + cabId, r.id, json);
    }

    protected void updateLeg(int cabId, Task t) {
        String json = "{\"Id\":" + t.id + ", \"Status\":\"" + t.status +"\"}";
        log("leg", t.id, json);
        saveJSON("PUT", "legs", "cab" + cabId, t.id, json);
    }

    protected Route getRoute(int cabId) {
        String json = getEntityAsJson("cab"+cabId, ROUTES);
        return getRouteFromJson(json);
    }

    private Route getRouteFromJson(String str) {
        //"{"id":114472,"status":"ASSIGNED",
        //  "legs":[{"id":114473,"fromStand":16,"toStand":12,"place":0,"status":"ASSIGNED"}]}"
        Map map = getMap(str, this.objectMapper);
        if (map == null) {
            return null;
        }
        int id = (int) map.get("Id");
        List<Map> legs = (List<Map>) map.get("Legs");
        List<Task> tasks = new ArrayList<>();
        for (Map m : legs) {
            tasks.add(new Task( (int) m.get("Id"),
                                (int) m.get("From"),
                                (int) m.get("To"),
                                (int) m.get("Place"),
                                (int) m.get("Dist")));
        }
        return new Route(id, tasks);
    }

    private List<Stop> getStopsFromJson(String str) {
        try {
            return objectMapper.readValue(str, new TypeReference<List<Stop>>(){});
        } catch (JsonProcessingException e) {
            logger.warning("getStopsFromJson fail: "+ e.getMessage());
            return null;
        }
    }

    private Cab getCabFromJson(String json) {
        Map map = getMap(json, this.objectMapper);
        if (map == null) {
            return null;
        }
        return new Cab( (int) map.get("Id"),
                        (int) map.get("Location"),
                        getCabStatus((String) map.get("Status")),
                        null);
    }

    protected Demand getOrderFromJson(String str) {
        if (str == null || str.startsWith("OK")) {
            return null;
        }
        Map map = getMapFromJson(str, this.objectMapper);
        if (map == null) {
            logger.info("getMapFromJson returned NULL, json:" + str);
            return null;
        }
        try {
            Map cab = (Map) map.get("Cab");
            return new Demand(  (int) map.get("Id"),
                                (int) map.get("From"),
                                (int) map.get("To"),
                                (int) map.get("Wait"),
                                (int) map.get("Loss"),
                                getOrderStatus((String) map.get("Status")),
                                (boolean) map.get("InPool"),
                                (int) cab.get("Id"),
                                (int) map.get("Eta"),
                                (int) map.get("Distance")
                            );
        } catch (NullPointerException npe) {
            logger.info("NPE in getMapFromJson, json:" + str);
            return null;
        }
    }

    private void setAuthentication(HttpURLConnection con, String user, String passwd) {
        String auth = user + ":" + passwd;
        String encodedAuth = Base64.getEncoder().encodeToString(auth.getBytes(StandardCharsets.UTF_8));
        String authHeaderValue = "Basic " + encodedAuth;
        con.setRequestProperty("Authorization", authHeaderValue);
    }

    protected String saveJSON(String method, String entity, String user, int rec_id, String json) {
        StringBuilder response = null;
        String urlStr = HOST + entity;
        try {
            response = callApi(urlStr, method, user, json);
        } catch (Exception e) {
            String msg = method + ", entity: " + entity + ", rec_id: " + rec_id + ", json: " + json;
            log(user, "Exception. method: " + msg + ", cause=" + e.getMessage() + "; " + e.getCause());
            try { // once more 
                response = callApi(urlStr, method, user, json);
            } catch (Exception e2) {
                log(user, "Exception2. method" + msg + ", cause=" + e2.getMessage() + "; " + e2.getCause());
            }
        }
        // we don't need any feedback // getCabFromJson(response.toString())
        return response == null ? null : response.toString();
    }

    private StringBuilder callApi(String urlStr, String method, String user, String json) throws IOException  {
        String password = user;
        URL url = new URL(urlStr);
        HttpURLConnection con = (HttpURLConnection) url.openConnection();
        con.setRequestMethod(method);
        con.setRequestProperty("Content-Type", "application/json");
        con.setRequestProperty("Accept", "application/json");
        con.setDoOutput(true);
        // Basic auth
        setAuthentication(con, user, password);

        try (OutputStream os = con.getOutputStream()) {
            byte[] input = json.getBytes(StandardCharsets.UTF_8);
            os.write(input, 0, input.length);
        }
        return getResponse(con);
    }

    private String getEntityAsJson(String user, String urlStr) {
        StringBuilder result = null;
        HttpURLConnection con = null;
        URL url = null; 
        try {
            url = new URL(HOST + urlStr); // assumption that one customer has one order
            // taxi_order will be updated with eta, cab_id and task_id when assigned
            con = (HttpURLConnection) url.openConnection();
            setAuthentication(con, user, user);
            result = getResponse(con);
        } catch(Exception e) { 
            log(user, "Exception. Url:" + urlStr + ", cause=" + e.getMessage() + "; " + e.getCause() + "; " + e.getStackTrace().toString());
            try {
                con = (HttpURLConnection) url.openConnection();
                setAuthentication(con, user, user);
                result = getResponse(con);
            } catch(Exception e2) { 
                log(user, "Exception2. Url:" + urlStr + ", cause=" + e2.getMessage() + "; " + e2.getCause() + "; " + e2.getStackTrace().toString());
            }            
        }
        //finally { con.disconnect(); }
        if (!ROUTES.equals(urlStr) && (result == null || result.toString().length() == 0)) {
            logger.info("http conn to " + urlStr+ " returned null or empty string");
        }
        return result == null ? null : result.toString();
    }

    private StringBuilder getResponse(HttpURLConnection con) throws IOException {
        StringBuilder response = new StringBuilder();
        try (BufferedReader br = new BufferedReader(new InputStreamReader(con.getInputStream(), "utf-8"))) {
            String responseLine = null;
            while ((responseLine = br.readLine()) != null) {
                response.append(responseLine.trim());
            }
        }
        return response;
    }

    private Map getMapFromJson(String str, ObjectMapper objectMapper) {
        if ("OK".equals(str)) { // PUT
            return null;
        }
        return getMap(str, objectMapper);
    }

    private Map<String,Object> getMap(String json, ObjectMapper objectMapper) {
        try {
            return objectMapper.readValue(json, HashMap.class);
        } catch (JsonProcessingException e) {
            logger.warning(e.getCause() + "; " +  e.getMessage());
            return null;
        }
    }

    private List<Map<String, Object>> getListOfMap(String json, ObjectMapper objectMapper) {
        try {
            return objectMapper.readValue(json, new TypeReference<>(){});
        } catch (JsonProcessingException e) {
            logger.warning(e.getCause() + "; " +  e.getMessage());
            return null;
        }
    }

    public static void waitSecs(int secs) {
        try { Thread.sleep(secs*1000); } catch (InterruptedException e) {} // one minute
    }

    public static void waitMins(int mins) {
        try { Thread.sleep((long)mins * 60 * 1000); 
        } catch (InterruptedException e) { } // one minute
    }

    protected void log(String msg, int cabId, int routeId) {
        logger.info(msg + ", cab_id=" + cabId + ", route_id=" + routeId + ",");
    }

    protected void log(String msg, int from, int to, int cabId, int taskId) {
        logger.info(msg + ", from=" + from + ", to=" + to + ", cab_id=" + cabId + ", leg_id=" + taskId + ",");
    }

    protected void log(String entity, int id, String json) {
        logger.info("Saving " + entity +"=" + id + ", JSON=" + json);
    }

    protected void log(String user, String msg) {
        logger.info("User: " + user + ", msg=" + msg);
    }

    protected void logCust(String msg, int custId, int orderId){
        logger.info(msg + ", cust_id=" + custId+ ", order_id=" + orderId +",");
    }

    protected void log(String msg, int custId, int orderId, int cabId){
        logger.info(msg + ", cust_id=" + custId+ ", order_id=" + orderId +", cab_id=" + cabId + ",");
    }

    public static CabStatus getCabStatus (String stat) {
        switch (stat) {
            case "ASSIGNED": return CabStatus.ASSIGNED;
            case "FREE":     return CabStatus.FREE;
            case "CHARGING": return CabStatus.CHARGING;
            default: return null;
        }
    }

    public static OrderStatus getOrderStatus (String stat) {
        if (stat == null) {
            return null;
        }
        switch (stat) {
            case "ASSIGNED":  return OrderStatus.ASSIGNED;
            case "ABANDONED": return OrderStatus.ABANDONED;
            case "ACCEPTED":  return OrderStatus.ACCEPTED;
            case "CANCELLED": return OrderStatus.CANCELLED;
            case "COMPLETED":  return OrderStatus.COMPLETED;
            case "PICKEDUP":  return OrderStatus.PICKEDUP;
            case "RECEIVED":  return OrderStatus.RECEIVED;
            case "REFUSED":   return OrderStatus.REFUSED;
            case "REJECTED":  return OrderStatus.REJECTED;
            default: return null;
        }
    }

    public enum RouteStatus {
        PLANNED,   // proposed by Pool
        ASSIGNED,  // not confirmed, initial status
        ACCEPTED,  // plan accepted by customer, waiting for the cab
        REJECTED,  // proposal rejected by customer(s)
        ABANDONED, // cancelled after assignment but before 'PICKEDUP'
        STARTED,
        COMPLETED
    }

    public enum OrderStatus {
        RECEIVED,  // sent by customer
        ASSIGNED,  // assigned to a cab, a proposal sent to customer with time-of-arrival
        ACCEPTED,  // plan accepted by customer, waiting for the cab
        CANCELLED, // cancelled before assignment
        REJECTED,  // proposal rejected by customer
        ABANDONED, // cancelled after assignment but before 'PICKEDUP'
        REFUSED,   // no cab available, cab broke down at any stage
        PICKEDUP,
        COMPLETED
    }

    public enum CabStatus {
        ASSIGNED,
        FREE,
        CHARGING, // out of order, ...
    }

    public static int getFromYaml(String path, String key) {
        try (BufferedReader bufferedReader = new BufferedReader(new InputStreamReader(new FileInputStream(path), "UTF-8"))) {
            String curLine;
            while ((curLine = bufferedReader.readLine()) != null){
                if (curLine.contains(key)) {
                    int idx = curLine.indexOf(':');
                    if (idx != -1) {
                        return Integer.parseInt(curLine.substring(idx+1).trim());
                    }
                    return -1;
                }
            }
        }
        catch (IOException ioe) { }
        return -1;
    }

    protected int getDistance(List<Stop> stops, int from, int to) {
        Stop f = stops.stream().filter(s -> from == s.id).findAny().orElse(null);
        Stop t = stops.stream().filter(s -> to == s.id).findAny().orElse(null);
        if (f == null || t == null) return -1; // ERR
        return (int) dist(f.latitude, f.longitude, t.latitude, t.longitude, 'K');
    }

    // https://dzone.com/articles/distance-calculation-using-3
    private double dist(double lat1, double lon1, double lat2, double lon2, char unit) {
        double theta = lon1 - lon2;
        double dist = Math.sin(deg2rad(lat1)) * Math.sin(deg2rad(lat2)) + Math.cos(deg2rad(lat1))
                    * Math.cos(deg2rad(lat2)) * Math.cos(deg2rad(theta));
        dist = Math.acos(dist);
        dist = rad2deg(dist);
        dist = dist * 60 * 1.1515;
        if (unit == 'K') {
        dist = dist * 1.609344;
        } else if (unit == 'N') {
        dist = dist * 0.8684;
        }
        return (dist);
    }

    /*:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::*/
    /*::  This function converts decimal degrees to radians             :*/
    /*:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::*/
    private double deg2rad(double deg) {
        return (deg * Math.PI / 180.0);
    }

    /*:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::*/
    /*::  This function converts radians to decimal degrees             :*/
    /*:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::*/
    private double rad2deg(double rad) {
        return (rad * 180.0 / Math.PI);
    }
}
