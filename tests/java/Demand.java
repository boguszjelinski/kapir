package org.kabina.kat;

import java.time.LocalDateTime;

public class Demand {
    public int id, from, to;
    public int eta; // set when assigned
    public boolean inPool;
    public int cab_id;
    public ApiClient.OrderStatus status;
    public int maxWait; // max wait for assignment
    public int maxLoss; // [%] loss in Pool
    public LocalDateTime atTime;
    public int distance;

    public Demand(int id, int from, int to, int wait, int loss, ApiClient.OrderStatus status, 
                 boolean inPool, int cab_id, int eta, int distance) {
        this.id = id;
        this.from = from;
        this.to = to;
        this.maxWait = wait;
        this.maxLoss = loss;
        this.status = status;
        this.inPool = inPool;
        this.cab_id = cab_id;
        this.eta = eta;
        this.distance = distance;
    }

    public Demand(int id, int from, int to, int wait, int loss, LocalDateTime atTime) {
        this.id = id;
        this.from = from;
        this.to = to;
        this.maxWait = wait;
        this.maxLoss = loss;
        this.atTime = atTime;
    }

    public void setStatus (ApiClient.OrderStatus stat) { this.status = stat; }
    public void setId(int id) { this.id = id; }
    public void setFrom(int fromStand) { this.from = fromStand; }
    public void setTo(int toStand) { this.to = toStand; }
    public void setEta(Integer eta) { this.eta = eta; }
    public void setInPool(Boolean inPool) { this.inPool = inPool; }
}