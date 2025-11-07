/*
 * Beacon

Saleem Bhatti <saleem@st-andrews.ac.uk>
Nov 2025, code check (sjm55).
Sep 2024, code check with java 21 on CS Linux Lab machines.
Sep 2022, code check.
Sep 2021, code check.
Sep 2020, code check.
Sep 2019, code check.
Oct 2018, initial version.

Send out a multicast beacon and listen out for other beacons.
*/

import java.io.UnsupportedEncodingException;
import java.text.SimpleDateFormat;
import java.util.Date;

class Beacon
{
  static Configuration     c; // config info
  static MulticastEndpoint m; // multicast socket + config info
  static String username = System.getProperty("user.name");

  final static int txRatio = 5; // reduce transmissions

  public static void main(String args[])
  {
    c = new Configuration("beacon.properties");
    m = new MulticastEndpoint(c);

    m.join();

    for (int b = c.beacon; b > 0; --b) {

      // simple tasks ... could run as a thread.
      if (b % txRatio == 0) txBeacon(); // send a beacon
      rxBeacon(); // check for incoming beacon

      // pause to avoid excess network traffic and CPU hogging
      try { Thread.sleep(c.sleepTime); }
      catch (InterruptedException e) { /* do nothing */ }

    } // for (b)

    m.leave();
  }


  /**
   * Receive a beacon.
   */
  static void rxBeacon()
  {

    try {
      byte[] b = new byte[c.msgSize];
      MulticastEndpoint.PktType p = m.rx(b);

      if (p == MulticastEndpoint.PktType.none) return;

      String logRequest = "";
      if (p == MulticastEndpoint.PktType.ip4) logRequest = "->rx4 ";
      if (p == MulticastEndpoint.PktType.ip6) logRequest = "->rx6 ";

      logRequest = logRequest + new String(b, "US-ASCII");
      logRequest = logRequest.trim();
      c.log.writeLog(logRequest, true);
    }

    catch (UnsupportedEncodingException e) {
      System.out.println("rxBeacon(): " + e.getMessage());
    }
  }


  /**
   * Transmit a beacon.
   */
  static void txBeacon()
  {
    SimpleDateFormat sdf = new SimpleDateFormat("yyyyMMdd-HHmmss.SSS");
    String now = sdf.format(new Date());
    String body = new String("|" + now +
                             "|" + username +
                             "|" + c.hostName +
                             "|");  

    byte[] b = null;

    try {
      String msg;

      msg = body + c.ipAddr4 + "|";
      b = msg.getBytes("US-ASCII");
      if (m.tx(MulticastEndpoint.PktType.ip4, b))
        c.log.writeLog("tx4-> " + msg, true);

      msg = body + c.ipAddr6 + "|";
      b = msg.getBytes("US-ASCII");
      if (m.tx(MulticastEndpoint.PktType.ip6, b))
        c.log.writeLog("tx6-> " + msg, true);
    }

    catch (UnsupportedEncodingException e) {
      System.out.println("txBeacon(): " + e.getMessage());
    }
  }

}
