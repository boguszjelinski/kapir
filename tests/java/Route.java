package org.kabina.kat;

import java.util.List;

public class Route {
    public int id;
    ApiClient.RouteStatus status;
    List<Task> tasks;
    public Route(int id, List<Task> tasks) { 
        this.id = id;
        this.tasks = tasks;
    }
    public List<Task> getTasks() { return tasks; }
    public void setTasks(List<Task> tasks) { this.tasks = tasks; }
    public void setId(int id) { this.id = id; }
    public int getId() { return id; }
}
