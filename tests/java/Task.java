package org.kabina.kat;

public class Task {
    //[{"id":114473,"fromStand":16,"toStand":12,"place":0,"status":"ASSIGNED"}]}" 
    public int id, fromStand, toStand, place, distance;
    public ApiClient.RouteStatus status;

    public Task(int id, int fromStand, int toStand, int place, int distance) {
        this.id = id;
        this.fromStand = fromStand;
        this.toStand = toStand;
        this.place = place;
        this.distance = distance;
    }
    public int getFromStand() { return this.fromStand; }
    public void setFromStand(int stand) { this.fromStand = stand; }
    public int getToStand() { return this.toStand; }
    public void setToStand(int stand) { this.toStand = stand; }
    public int getPlace() { return place; }
    public void setPlace(int order) { this.place = order; }

    public int getDistance() {
        return distance;
    }

    public void setDistance(int distance) {
        this.distance = distance;
    }
}