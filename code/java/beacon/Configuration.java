/*
 * Beacon - Configuration information using Java Properties.

Saleem Bhatti <saleem@st-andrews.ac.uk>
Nov 2025, code check (sjm55).
Sep 2024, code check with java 21 on CS Linux Lab machines.
Sep 2022, updated code for dual-stack IPv4/IPv6.
Sep 2021, updated code to remove deprecated API usage in Java 17 LTS.
Sep 2020, code check.
Sep 2019, code check.
Oct 2018, initial version.

Send out a multicast beacon and listen out for other beacons.
*/

/*
This is an object that gets passed around, containing useful information.
*/

import java.io.IOException;
import java.io.InputStream;
import java.net.InetAddress;
import java.net.NetworkInterface;
import java.net.UnknownHostException;
import java.util.Enumeration;
// https://docs.oracle.com/en/java/javase/17/docs/api/java.base/java/util/Properties.html
import java.util.Properties;

public class Configuration
{
  public LogFileWriter log;
  public Properties    properties;
  // The following can be overridden in the constructor argument
  public String        propertiesFile = "configuration.properties";

  /*
  The assumption is that the workstation only has a single main interface.
  The "first" detected network interface is used.

  This is simplistic, but fine for the CS lab linux machines.

  hostName, nifName, ipAddr4, and ipAddr6 are detected dynamically.
  */
  public String  hostName = "";
  public String  nifName = "";
  public String  ipAddr4 = "";
  public String  ipAddr6 = "";
  public NetworkInterface nif = null;

  // These default values can be overridden by values in the properties file.
  public String  logFile = "configuration.log";
  public String  mAddr4 = "239.0.98.76"; // group address
  public String  mAddr6 = "ff02::9876:9876"; // group address
  public final String zeroAddr = "0"; // to indicate a "null" address

  public int     mPort = 9876; // group port
  public int     mTTL = 2; // plenty for the lab
  public int     soTimeout = 1; // ms
  public boolean reuseAddr = true; // allow address use by other apps
  public boolean loopback = false; // do not see my own transmissions
  /*
  Note that "loopback" should be "disable loopback". On Java 21
  in Linux on the School Lab machines, this works with the 
  opposite sense. For example, if you run this code on macOS,
  then you need to set it to "true" to avoid seeing your own
  packets.
  */

  // application config : defaults, can be overridden in properties file
  public int     msgSize = 128; // bytes
  public int     sleepTime = 10000; // ms
  public int     beacon = 10; // how many to send

  // This should not be loaded from a config file, of course.
  public String  hostInfo;

  /**
   * @param propertiesFile : name of the file to load
   */
  Configuration(String propertiesFile)
  {
    if (propertiesFile != null) { this.propertiesFile = propertiesFile; }

    String nullString = "null";

    try {

      /*
      This is *not* a general way of reliably discovering all the local IPv4
      and IPv6 addresses being used by a host on all of its interfaces, and
      working out which is the "main" interface. However, it works for the CS
      lab linux machines, which have:
      1. a single gigabit ethernet interface.
      2. a single IPv4 address for that interface.
      3. only link-local IPv6 (no global IPv6 prefix).
      */
      InetAddress ip4 = InetAddress.getLocalHost(); // assumes IPv4!
      hostName = ip4.getHostName();
      ipAddr4 = ip4.getHostAddress(); // assumes IPv4!
      nif = NetworkInterface.getByInetAddress(ip4); // assume the "main" interface
      nifName = nif.getName();
      Enumeration<InetAddress> e_addr = nif.getInetAddresses();
      while(e_addr.hasMoreElements()) {
        final InetAddress a = e_addr.nextElement();
        if (a.isLinkLocalAddress()) { // assume only this will be used
          // will include interface name, e.g. fe80:0:0:0:1067:14a1:4e8b:28ac%en0 
          ipAddr6 = a.getHostAddress(); 
          break ;
        }
      }

      hostInfo = hostName + " " + nifName + " " + ipAddr4 + " " + ipAddr6;
      System.out.println("** detected: " + hostInfo);

      properties = new Properties();
      InputStream p = getClass().getClassLoader().getResourceAsStream(propertiesFile);
      if (p != null) {
        properties.load(p);
        String s;

        if ((s = properties.getProperty("logFile")) != null) {
          System.out.println(propertiesFile + " logFile: " + logFile + " -> " + s);
          logFile = new String(s);
        }

        if ((s = properties.getProperty("loopback")) != null) {
          System.out.println(propertiesFile + " loopback: " + loopback + " -> " + s);
          loopback = Boolean.valueOf(s);
        }

        if ((s = properties.getProperty("mAddr4")) != null) {
          System.out.println(propertiesFile + " mAddr4: " + mAddr4 + " -> " + s);
          mAddr4 = s.equalsIgnoreCase(nullString) ? null : new String(s);
        }

        if ((s = properties.getProperty("mAddr6")) != null) {
          System.out.println(propertiesFile + " mAddr6: " + mAddr6 + " -> " + s);
          mAddr6 = s.equalsIgnoreCase(nullString) ? null : new String(s);
        }

        if ((s = properties.getProperty("mPort")) != null) {
          System.out.println(propertiesFile + " mPort: " + mPort + " -> " + s);
          mPort = Integer.parseInt(s);
        }

        if ((s = properties.getProperty("mTTL")) != null) {
          System.out.println(propertiesFile + " mTTL: " + mTTL + " -> " + s);
          mTTL = Integer.parseInt(s);
        }

        if ((s = properties.getProperty("reuseAddr")) != null) {
          System.out.println(propertiesFile + " reuseAddr: " + reuseAddr + " -> " + s);
          reuseAddr = Boolean.valueOf(s);
        }

        if ((s = properties.getProperty("soTimeout")) != null) {
          System.out.println(propertiesFile + " soTimeout: " + soTimeout + " -> " + s);
          soTimeout = Integer.parseInt(s);
        }

        /*
         * Application-specific configuration
         */
        if ((s = properties.getProperty("msgSize")) != null) {
          System.out.println(propertiesFile + " msgSize: " + msgSize + " -> " + s);
          msgSize = Integer.parseInt(s);
        }

        if ((s = properties.getProperty("sleepTime")) != null) {
          System.out.println(propertiesFile + " sleepTime: " + sleepTime + " -> " + s);
          sleepTime = Integer.parseInt(s);
        }

        if ((s = properties.getProperty("beacon")) != null) {
          System.out.println(propertiesFile + " beacon: " + beacon + " -> " + s);
          beacon = Integer.parseInt(s);
        }

        p.close();
      }

      log = new LogFileWriter(logFile);
      log.writeLog("-* Detected: " + hostInfo, true);
      log.writeLog("-* logFile=" + logFile, true);
      log.writeLog("-* mAddr4=" + mAddr4, true);
      log.writeLog("-* mAddr6=" + mAddr6, true);
      log.writeLog("-* mPort=" + mPort, true);
      log.writeLog("-* mTTL=" + mTTL, true);
      log.writeLog("-* loopback=" + loopback, true);
      log.writeLog("-* reuseAddr=" + reuseAddr, true);
      log.writeLog("-* soTimeout=" + soTimeout, true);
      log.writeLog("-* msgSize=" + msgSize, true);
      log.writeLog("-* sleepTime=" + sleepTime, true);
      log.writeLog("-* beacon=" + beacon, true);
    }

    catch (UnknownHostException e) {
      System.out.println("Problem: " + e.getMessage());
    }

    catch (NumberFormatException e) {
      System.out.println("Problem: " + e.getMessage());
    }

    catch (IOException e) {
      System.out.println("Problem: " + e.getMessage());
    }

  }
}
