package org.kabina.kat;

public class Stop {
    public int id;

    public String no;
    public String name;
    public String type;
    public int bearing;
    public double latitude;
    public double longitude;

    public Stop() {}

    public Stop(int id, int bearing, double latitude, double longitude, String name) {
        this.id = id;
        this.name = name;
        this.bearing = bearing;
        this.latitude = latitude;
        this.longitude = longitude;
    }

    public Stop(int id, String no, String name, String type, int bearing, double latitude, double longitude) {
        this.id = id;
        this.no = no;
        this.name = name;
        this.type = type;
        this.bearing = bearing;
        this.latitude = latitude;
        this.longitude = longitude;
    }

    public int getId() {
        return id;
    }

    public void setId(int id) {
        this.id = id;
    }

    public String getNo() {
        return no;
    }

    public void setNo(String no) {
        this.no = no;
    }

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public String getType() {
        return type;
    }

    public void setType(String type) {
        this.type = type;
    }

    public int getBearing() {
        return bearing;
    }

    public void setBearing(int bearing) {
        this.bearing = bearing;
    }

    public double getLatitude() {
        return latitude;
    }

    public void setLatitude(double latitude) {
        this.latitude = latitude;
    }

    public double getLongitude() {
        return longitude;
    }

    public void setLongitude(double longitude) {
        this.longitude = longitude;
    }
}
