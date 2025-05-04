package org.kabina.kat;

import java.io.BufferedReader;
import java.io.FileReader;
import java.io.IOException;
import java.time.LocalDateTime;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;
import java.util.concurrent.TimeUnit;
import java.util.logging.FileHandler;
import java.util.logging.Logger;
import java.util.logging.SimpleFormatter;

public class Main {
    static final int MAX_CABS = 5000;
    static final int MAX_CUST = 600000;
    static final int MAX_STOP = 5192;
    static final int DURATION =  30; // min
    static final int REQ_PER_MIN = 1000;
    static final int MAX_WAIT = 15;
    static final int MAX_POOL_LOSS = 70;
    static final int AT_TIME_LAG = 30;
    static Random rand;

    public static void main(String[] args) throws InterruptedException {

        Thread cabThread[] =  new Thread[MAX_CABS];
        final Logger cabLogger = Logger.getLogger( "org.kabina.cabgenerator" );
        configureLogger(cabLogger, "cab.log");
        for (int i=0; i<MAX_CABS; i++) {
            final int id = i;
            cabThread[i] = Thread.startVirtualThread(() -> {
                CabRunnable cab = new CabRunnable(id, id * (int)(MAX_STOP/MAX_CABS), cabLogger);
                cab.live();
            });
            Thread.sleep(1); // so that to disperse them a bit and not to kill backend
        }
        // wait a minute for cabs to stand up
        Thread.sleep(60*1000);

        final Logger custLogger = Logger.getLogger( "org.kabina.customergenerator" );
        configureLogger(custLogger, "customer.log");
        Thread custThread[] =  new Thread[MAX_CUST];
        List<Integer[]> orders = readOrders();
        int id = 0;
        for (int t = 0; t < DURATION; t++) { // time axis
            // filter out demand for this time point
            for (int i = 0; i < REQ_PER_MIN; i++) {
                // to make common trips more probable
                LocalDateTime atTime = null;
                /*if (orders.get(id)[0] % 3 == 0 && t < DURATION - AT_TIME_LAG) {
                    atTime = LocalDateTime.now().plusMinutes(AT_TIME_LAG);
                }*/
                final Demand d = new Demand(id,
                        orders.get(id)[0],
                        orders.get(id)[1], // to
                        MAX_WAIT,
                        MAX_POOL_LOSS,
                        atTime
                );
                custThread[id++] = Thread.startVirtualThread(() -> {
                    CustomerRunnable cust = new CustomerRunnable(d, custLogger);
                    cust.live();
                });
                Thread.sleep(30*1000/REQ_PER_MIN); // should be 60 but creating a thread takes time
            }
            TimeUnit.SECONDS.sleep(60);
        }
        for (int i=0; i<MAX_CABS; i++) cabThread[i].join();
        for (int i=0; i<id; i++) custThread[i].join();
    }

    public static void configureLogger(Logger loggr, String file) {
        System.setProperty("java.util.logging.SimpleFormatter.format",
                           "%1$tY-%1$tm-%1$td %1$tH:%1$tM:%1$tS %4$-6s %5$s%6$s%n"); // %2$s - class
        FileHandler fh;
        try {
            fh = new FileHandler(file);
            loggr.addHandler(fh);
            SimpleFormatter formatter = new SimpleFormatter();
            fh.setFormatter(formatter);
        } catch (SecurityException | IOException e) {
            e.printStackTrace();
        }
    }

    static List<Integer[]> readOrders() {
        List<Integer[]> records = new ArrayList<>();
        try (BufferedReader br = new BufferedReader(
                new FileReader("../orders.txt"))) {
            String line;
            while ((line = br.readLine()) != null) {
                String[] values = line.split(",");
                Integer[] ints = new Integer[2];
                ints[0]= Integer.valueOf(values[0]);
                ints[1]= Integer.valueOf(values[1]);
                records.add(ints);
            }
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
        return records;
    }
}
