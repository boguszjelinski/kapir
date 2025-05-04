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

import java.util.Collections;
import java.util.List;
import java.util.logging.Logger;

public class CabRunnable extends ApiClient {

    static final int MAX_TIME = 70; // minutes
    static final int CHECK_INTERVAL = 15; // secs

    private final int cab_id;
    private final int stand;

    public CabRunnable(int id, int stand, Logger logger) {
        this.cab_id = id;
        this.stand = stand;
        this.logger = logger;
    }

    public void live() {
        logger.info("Starting my shift, cab_id=" + cab_id);
        /*
            1. check if any valid route
            2. if not - wait 30sec
            3. if yes - get the route with tasks (legs), mark cab as 'not available' (sheduler's job)
            4. execute route - go from stand to stand, "go" = wait this ammount of time, virtual trip
            5. report stoping at stands (Kaboot must notify customers)
            6. report 'cab is free'
        */
        // let's begin with cab's location
        Cab cab = getCabAsCab("cabs/", cab_id, cab_id);
        if (cab == null) { 
            // cab should be activated by an admin
            logger.warning("cab_id=" +  cab_id + " not activated");
            return;
        }

        // non-random, nice deterministic distribution so that simulations are reproducible
        logger.info("Updating cab_id=" + cab_id + ", free at: " + stand);
        String json = "{\"Id\":" + cab_id + ", \"Location\":" + stand + ", \"Status\": \""+ CabStatus.FREE +"\"}";
        saveJSON("PUT", "cabs", "cab" + cab_id, cab_id, json);
        cab.location = stand; // the cab location read from DB (see above) might be wrong, that was the day before e.g.

        for (int t = 0; t < MAX_TIME * (60/CHECK_INTERVAL); t++) { 
            Route route = getRoute(cab_id); // TODO: status NULL
            if (route != null && route.getId() > -1) { // this cab has been assigned a job
                log("New route to run", cab_id,  route.getId());
                // go to first task - virtual walk, just wait this amount of time
                List<Task> legs = route.getTasks();
                if (legs == null || legs.isEmpty()) {
                    log("Error - route has no legs", cab_id, route.getId());
                    continue;
                }
                // sorting legs by place 
                // TODO: backend should give a sorted list
                Collections.sort(legs, (Task t1, Task t2) -> t1.place - t2.place);
                cab.status = CabStatus.ASSIGNED;
                Task task = legs.get(0);
                // the first leg in a route should already be cab's move to pickup the first customer
                // but let's check
                if (task.fromStand != cab.location) { 
                    // strange - scheduler did not know cab's location (! never found in logs)
                    log("Error, first leg does not start at cabs location. Moving",
                            cab.location, task.fromStand, cab_id, + task.id);
                    waitMins(task.distance); // cab is moving
                    cab.location = task.fromStand;
                    // inform that cab is at the stand -> update Cab entity, 'completed' previous Task
                    updateCab(cab.id, cab);
                }
                deliverPassengers(legs, cab);
                route.status = RouteStatus.COMPLETED;
                updateRoute(cab.id, route);
            } 
            waitSecs(CHECK_INTERVAL);
        }
    }
  
    private void deliverPassengers(List<Task> legs, Cab cab) {
        Route route = null;
        for (int i=0; i < legs.size(); i++) {
            waitMins(1); // wait 1min: pickup + dropout; but it is stupid if the first leg has no passenger!!
            // go from where you are to task.stand
            Task task = legs.get(i);
            log("Moving", task.fromStand, task.toStand, this.cab_id, task.id);
            task.status = RouteStatus.STARTED;
            updateLeg(cab.id, task);
            waitMins(task.distance); // cab is moving
            task.status = RouteStatus.COMPLETED;
            updateLeg(cab.id, task);
            cab.location = task.toStand;
            // inform sheduler / customer
            if (i == legs.size() - 1) {
                cab.status = CabStatus.FREE;
            }
            updateCab(cab.id, cab); // such call should 'completed' tasks; at the last task -> 'complete' route and 'free' that cab
            // !! update leg here -> completed
            // a route can be extended with new legs (but only these 'not started'), we have to read it again
            route = getRoute(cab.id);
            legs = route.getTasks();
            Collections.sort(legs, (Task t1, Task t2) -> t1.place - t2.place);
        }
    }
}
