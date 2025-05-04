package org.kabina.kat;

public class Cab {
    public Cab(int i, int l, ApiClient.CabStatus s, String n) {
        this.id = i;
        this.location = l;
        this.status = s;
        this.name = n;
    }
    public int id;
    public int location;
    public ApiClient.CabStatus status;
    public String name;

    public void setId(int id) { this.id = id; }
    public void setLocation(int l) { this.location = l; }
    public void setStatus(ApiClient.CabStatus s) { this.status = s; }
    public void setName(String name) {
        this.name = name;
    }
}
