/*
  Application Configuration information
  CS4105 Practical P2 - Discover and Share

  Saleem Bhatti
  Nov 2025, code check (sjm).
  18 Sep 2024, checked with java 21 on CS Lab Linux machines. 
  Oct 2023, Oct 2022, Oct 2021, Oct 2020, Sep 2019, Oct 2018

*/

/*
  This is an object that gets passed around, containing useful information.
*/

import java.io.IOException;
import java.io.InputStream;
import java.net.InetAddress;
import java.net.UnknownHostException;
//https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/Properties.html
import java.util.Properties;

public class Configuration
{
  // Everything here is "public" to allow tweaking from user code.
  public Properties    properties;
  public String        propertiesFile = "filetreebrowser.properties";
  public LogFileWriter log;
  public String        logFile = "filetreebrowser.log";

  // These default values can be overriden in the properties file.

  // 'id -u' gives a numeric uid, u, which will be unique within the lab.
  // You can construct your "personal" multicast address, by "splitting"
  // `u` across the lower 32 bits. For example, if `u` is 414243,
  // mAddr6 = ff02::41:4243
  // and your "personal" port number, mPort_ = u.
  public String  mAddr6 = "ff02::4105:4105"; // CS4105 whole class group
  public int     mPort = 4105;

  public int     mTTL = 2; // plenty for the lab
  public boolean loopback = true; // ignore my own transmissions
  public boolean reuseAddr = false; // allow address use by other apps
  public int     soTimeout = 1; // ms
  public int     sleepTime = 5000; // ms

  // // // //
  // application config -- default values
  public String  rootDir = "root_dir"; // sub-dir in current dir
  public String  id; // System.getProperty("user.name") @ fqdn;
  public int     maximumMessageSize = 500; // bytes
  public int     maximumAdvertisementPeriod = 1000; // ms

  public Boolean checkOption(String value, String[] optionList) {
    boolean found = false;
    for (String option : optionList) {
      if (value.equals(option)) { found = true; break; }
    }
    return found;
  }

  public String[] true_false = {"true", "false"}; // Could have used enum.
  public String[] searchOptions = // Could have used enum.
         {"none", "path", "path-filename", "path-filename-substring"};
  public String  search = "none"; // from searchOptions_
  public boolean download = false;

  // these should not be loaded from a config file, of course

  Configuration(String file) 
  {
    if (file != null) { propertiesFile = file; }

    String h;

    try {
      // h = InetAddress.getLocalHost().getHostName();
      h =  InetAddress.getLocalHost().getCanonicalHostName();
    }
    catch (UnknownHostException e) {
      System.out.println("Problem: " + e.getMessage());
      h = "FileTreeBrowser-host";
      System.out.println("Unknown host name: using " + h);
    }

    try {
      id = new String(System.getProperty("user.name") + "@" + h);
      logFile = new String(id + "-log.log");

      properties = new Properties();
      InputStream p = getClass().getClassLoader().getResourceAsStream(propertiesFile);
      if (p != null) {
        properties.load(p);
        String s;

        if ((s = properties.getProperty("logFile")) != null) {
          System.out.println(propertiesFile + " logFile: " + logFile + " -> " + s);
          logFile = new String(s);
        }

        if ((s = properties.getProperty("id")) != null) {
          System.out.println(propertiesFile + " id: " + id + " -> " + s);
          id = new String(s + "@" + h);
        }

        if ((s = properties.getProperty("rootDir")) != null) {
          System.out.println(propertiesFile + " rootDir: " + rootDir + " -> " + s);
          rootDir = new String(s);
        }

        if ((s = properties.getProperty("mAddr6")) != null) {
          System.out.println(propertiesFile + " mAddr6: " + mAddr6 + " -> " + s);
          mAddr6 = new String(s);
          // should check for valid mutlicast address range
        }

        if ((s = properties.getProperty("mPort")) != null) {
          System.out.println(propertiesFile + " mPort: " + mPort + " -> " + s);
          mPort = Integer.parseInt(s);
          // should check for valid port number range
        }

        if ((s = properties.getProperty("mTTL")) != null) {
          System.out.println(propertiesFile + " mTTL: " + mTTL + " -> " + s);
          mTTL = Integer.parseInt(s);
          // should check for valid TTL number range
        }

        if ((s = properties.getProperty("loopback")) != null) {
          if (!checkOption(s, true_false)) {
            System.out.println(propertiesFile + " bad value for 'loopback': '" + s + "' -> using 'false'");
            s = new String("false");
          }
          System.out.println(propertiesFile + " loopback: " + loopback + " -> " + s);
          loopback = Boolean.valueOf(s);
        }

        if ((s = properties.getProperty("reuseAddr")) != null) {
          if (!checkOption(s, true_false)) {
            System.out.println(propertiesFile + " bad value for 'reuseAddr': '" + s + "' -> using 'false'");
            s = new String("false");
          }
          System.out.println(propertiesFile + " reuseAddr: " + reuseAddr + " -> " + s);
          reuseAddr = Boolean.valueOf(s);
        }

        if ((s = properties.getProperty("soTimeout")) != null) {
          System.out.println(propertiesFile + " soTimeout: " + soTimeout + " -> " + s);
          soTimeout = Integer.parseInt(s);
          // should check for "sensible" timeout value
        }

        if ((s = properties.getProperty("sleepTime")) != null) {
          System.out.println(propertiesFile + " sleepTime: " + sleepTime + " -> " + s);
          sleepTime = Integer.parseInt(s);
          // should check for "sensible" sleep value
        }

        if ((s = properties.getProperty("maximumMessageSize")) != null) {
          System.out.println(propertiesFile + " maximumMessageSize: " + maximumMessageSize + " -> " + s);
          maximumMessageSize = Integer.parseInt(s);
          // should check for "sensible" message size value
        }

        if ((s = properties.getProperty("maximumAdvertisementPeriod")) != null) {
          System.out.println(propertiesFile + " maximumAdvertisementPeriod: " + maximumAdvertisementPeriod + " -> " + s);
          maximumAdvertisementPeriod = Integer.parseInt(s);
          // should check for "sensible" period value
        }

        if ((s = properties.getProperty("search")) != null) {
          if (!checkOption(s, searchOptions)) {
            System.out.println(propertiesFile + " bad value for 'search': '" + s + "' -> using 'none'");
            s = new String("none");
          }
          System.out.println(propertiesFile + " search: " + search + " -> " + s);
          search = new String(s);
        }

        if ((s = properties.getProperty("download")) != null) {
          if (!checkOption(s, true_false)) {
            System.out.println(propertiesFile + " bad value for 'download': '" + s + "' -> using 'false'");
            s = new String("false");
          }
          System.out.println(propertiesFile + " download: " + download + " -> " + s);
          download = Boolean.parseBoolean(s);
        }

        p.close();
      }

      log = new LogFileWriter(logFile);
      log.writeLog("-* logFile=" + logFile, true);
      log.writeLog("-* id=" + id, true);
      log.writeLog("-* rootDir=" + rootDir, true);
      log.writeLog("-* mAddr6=" + mAddr6, true);
      log.writeLog("-* mPort=" + mPort, true);
      log.writeLog("-* mTTL=" + mTTL, true);
      log.writeLog("-* loopback=" + loopback, true);
      log.writeLog("-* reuseAddr=" + reuseAddr, true);
      log.writeLog("-* soTimeout=" + soTimeout, true);
      log.writeLog("-* sleepTime=" + sleepTime, true);
      log.writeLog("-* maximumMessageSize=" + maximumMessageSize, true);
      log.writeLog("-* maximumAdvertisementPeriod=" + maximumAdvertisementPeriod, true);
      log.writeLog("-* search=" + search, true);
      log.writeLog("-* download=" + download, true);
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
