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

import java.util.logging.Logger;
import static java.lang.StrictMath.abs;

import java.time.Duration;
import java.time.LocalDateTime;

class CustomerRunnable extends ApiClient {
   
    static final int MAX_WAIT_FOR_RESPONSE = 3; // minutes, more would mean a serious configuration error. 1min is the goal
    static final int CHECK_INTERVAL = 30; // secs, not to kill the backend
    static final int MAX_TRIP_LOSS = 2; // we need a time delay before we will report a serious error in the whole simulation
    static final int MAX_TRIP_LEN = 30; // see MAX_TRIP in CustomerGenerator, 30 means that something went terribly wrong, 
    static final int AT_TIME_LAG = 3; // at-time-lag in application.YML
    private final Demand tOrder;
    
    public CustomerRunnable(Demand o, Logger logger) {
        this.tOrder = o;
        this.tOrder.setStatus(OrderStatus.RECEIVED);
        this.logger = logger;
    }

    public void live() {
        /*
            1. request a cab
            2. wait for an assignment - do you like it ?
            3. wait for a cab
            4. take a trip
            5. mark the end
        */
        // send to dispatcher that we need a cab
        // order id returned
        int custId = tOrder.id;
        logger.info("Request cust_id=" + custId + ", from=" + tOrder.from + ", to=" +tOrder.to);
        Demand order = saveOrder("POST", tOrder, custId); //          order = d; but now ith has entity id
        if (order == null) {
            logger.info("Unable to request a cab, cust_id=" + custId);
            return;
        }
        int orderId = order.id;

        logCust("Cab requested", custId, orderId);
        // just give kaboot a while to think about it
        // pool ? cab? ETA ?

        if (tOrder.atTime != null) {
            Duration duration = Duration.between(LocalDateTime.now(), tOrder.atTime);
            int wait = (int) (duration.getSeconds() - AT_TIME_LAG * 60);
            if (wait > 0) {
                waitSecs(wait);
            }
        } 
        
        waitSecs(30); // just give the solver some time
        
        order = waitForAssignment(custId, orderId);
        
        if (order == null || order.status != OrderStatus.ASSIGNED //|| ord.cab_id == -1
           ) { // Kaboot has not answered, too busy
            // complain
            if (order == null) {
                logCust("Waited in vain, no answer", custId, orderId);
            } else {
                logCust("Waited in vain, no assignment", custId, orderId);
                order.status = OrderStatus.CANCELLED; // just not to kill scheduler
                saveOrder("PUT", order, custId); 
            }
            return;
        }

        log("Assigned", custId, orderId, order.cab_id);
        
        if (order.eta > order.maxWait) {
            // TASK: stop here, now only complain
            logCust("ETA exceeds maxWait", custId, orderId);
        }
        
        order.status = OrderStatus.ACCEPTED;
        saveOrder("PUT", order, custId); // PUT = update
        log("Accepted, waiting for that cab", custId, orderId, order.cab_id);
        
        if (!hasArrived(custId, order.cab_id, tOrder.from, tOrder.maxWait)) {
            // complain
            log("Cab has not arrived", custId, orderId, order.cab_id);
            order.status = OrderStatus.CANCELLED; // just not to kill scheduler
            saveOrder("PUT", order, custId); 
            return;
        }
       
        takeATrip(custId, order); 
      
        if (order.status != OrderStatus.COMPLETED) {
            order.status = OrderStatus.CANCELLED; // just not to kill scheduler
            log("Status is not COMPLETED, cancelling the trip", custId, orderId, order.cab_id);
            saveOrder("PUT", order, custId); 
        }
    }

    private Demand waitForAssignment(int custId, int orderId) {
        Demand order = null;
        for (int t = 0; t < MAX_WAIT_FOR_RESPONSE * (60 / CHECK_INTERVAL); t++) {
            order = getOrder(custId, orderId);
            if (order == null) {
                logCust("Serious error, order not found", custId, orderId);
                return null;
            }
            if (order.status == OrderStatus.ASSIGNED)  {
                break;
            }
            waitSecs(CHECK_INTERVAL); 
        }
        return order;
    }

    private boolean hasArrived(int custId, int cabId, int from, int wait) {
        for (int t = 0; t < wait * (60/CHECK_INTERVAL) ; t++) { // *4 as 15 secs below
            waitSecs(CHECK_INTERVAL);
            Cab cab = getCab("cabs/", custId, cabId);
            if (cab.location == from) {
                return true;
            }
        }
        return false;
    }

    private void takeATrip(int custId, Demand order) {
        // authenticate to the cab - open the door?
        log("Picked up", custId, order.id, order.cab_id);
        order.status = OrderStatus.PICKEDUP;
        saveOrder("PUT", order, custId); // PUT = update

        int duration = 0; 
       
        for (; duration<MAX_TRIP_LEN * (60/CHECK_INTERVAL); duration++) {
            waitSecs(CHECK_INTERVAL);
            /*order = getEntity("orders/", cust_id, order_id);
            if (order.status == OrderStatus.COMPLETED && order.cab_id != -1)  {
                break;
            }*/
            Cab cab = getCab("cabs/", custId, order.cab_id);
            if (cab.location == order.to) {
                log("Arrived at " + order.to, custId, order.id, order.cab_id);
                order.status = OrderStatus.COMPLETED;
                saveOrder("PUT", order, custId); 
                break;
            }
        }
          
        if (duration >= MAX_TRIP_LEN * (60.0/CHECK_INTERVAL)) {
            log("Something wrong - customer has never reached the destination", custId, order.id, order.cab_id);
        } else {
            if (order.inPool) {
                if (duration/(60.0/CHECK_INTERVAL) > order.distance * (1.0+ (order.maxLoss/100.0)) + MAX_TRIP_LOSS) {
                    // complain
                    String str = " - duration: " + duration/(60/CHECK_INTERVAL) 
                                + ", distance: " + order.distance
                                + ", maxLoss: " + order.maxLoss
                                + ", " + duration/(60/CHECK_INTERVAL) + ">" + (int) (order.distance * (1+ (order.maxLoss/100)) + MAX_TRIP_LOSS);
                    log("Duration in pool was too long" + str, custId, order.id, order.cab_id);
                }
            } else { // not a carpool
                if (duration/(60.0/CHECK_INTERVAL) > order.distance + MAX_TRIP_LOSS) {
                    // complain
                    String str = " - duration: " + duration/(60/CHECK_INTERVAL) 
                                + ", distance: " + order.distance
                                + ", " + duration/(60/CHECK_INTERVAL) + ">" + (int) (order.distance + MAX_TRIP_LOSS);
                    log("Duration took too long" + str, custId, order.id, order.cab_id);
                }
            }
        }
    }
}
