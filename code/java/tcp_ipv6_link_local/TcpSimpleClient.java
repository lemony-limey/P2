/**
 * Simple, non-blocking TCP client, IPv4 or IPv6,
 * with input from keyboard, using socket timeout.
 *
 * saleem, 11 Oct 2023
 * sjm55 (checked, Nov 2025)
 */

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.net.InetAddress;
import java.net.NetworkInterface;
import java.net.Socket;
import java.net.SocketTimeoutException;
import java.net.UnknownHostException;
import java.util.Enumeration;

public class TcpSimpleClient {
  static int sleepTime_ = 5000; // milliseconds
  static int bufferSize_ = 80; // a line
  static int soTimeout_ = 10; // milliseconds

  public static void main(String[] args) {
    if (args.length != 2) { // user has not provided arguments
      System.out.println("\n TcpClient <server-ipv6-link-local-address> <portnumber> \n");
      System.out.println("   <server-ipv6-link-local-address> should not include the interface name:");
      System.out.println("   the local interface name will be detected automatically.");
      System.exit(0);
    }

    try {
      Socket       connection;
      InputStream  rx, kbd;
      OutputStream tx;
      byte[]       buffer;
      int          b ;

      connection = startClient(args[0], args[1]);
      kbd = System.in; // keyboard
      rx = connection.getInputStream();
      tx = connection.getOutputStream();
      b = 0;

      System.out.print("You have " + sleepTime_ + " milliseconds to type something -> ");
      Thread.sleep(sleepTime_); // wait

      buffer = new byte[bufferSize_];

      if (kbd.available() > 0) {
        b = kbd.read(buffer); // from keyboard

        if (b > 0) {
          tx.write(buffer, 0, b); // send to server
          System.out.println("Sending " + b + " bytes");
        }

        System.out.println("Waiting " + sleepTime_ + " milliseconds ...");
        Thread.sleep(sleepTime_); // wait for server response

        b = rx.read(buffer); // from server

        if (b > 0) { // something received from server
          String s = new String(buffer); /// assume it is a printable string
          System.out.println("Received " + b + " bytes --> " + s);
        }
        else if (b ==  0) System.out.println("Nothing from server -- giving up!");
        else if (b == -1) System.out.println("<Server closed connection>");

      } // if (System.in.available() > 0)

      else System.out.println("<Too slow!>");

      System.out.println("<Closing connection.>");
      connection.close();

    } // try

    catch (SocketTimeoutException e) {
      // no incoming data - just ignore
    }
    catch (InterruptedException e) {
      System.err.println("InterruptedException: " + e.getMessage());
    }
    catch (IOException e) {
      System.err.println("IOException: " + e.getMessage());
    }
  
  } // main()

  static Socket startClient(String hostname, String portnumber) {

    String ifName = "";

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
      InetAddress ip6 = InetAddress.getByName("::1"); // IPv6 localhost addr
      NetworkInterface nif = NetworkInterface.getByInetAddress(ip4); // assume "main" interface
      ifName = new String(nif.getName());
      Enumeration<InetAddress> e_addr = nif.getInetAddresses();
      InetAddress addr = ip6; // to avoid uninitialised variable warning
      while(e_addr.hasMoreElements()) {
        // final InetAddress a = e_addr.nextElement();
        addr = e_addr.nextElement();
        if (addr.isLinkLocalAddress()) { // assume only this will be used
          // will include interface name, e.g. fe80:0:0:0:1067:14a1:4e8b:28ac%en0 
          break ;
        }
      }
      ip6 = addr; // should be the link-local address
      System.out.println("--* Interface: " + ifName);
      System.out.println("--* IPv6 link-local address: " + ip6.toString());
    }

    catch (UnknownHostException e) {
      System.out.println("UnknownHostException: " + e.getMessage());
    }

    catch (IOException e) {
      System.err.println("IOException: " + e.getMessage());
    }

    Socket connection = null;

    try {
      InetAddress address;
      int         port;

      address = InetAddress.getByName(hostname + "%" + ifName);
      port = Integer.parseInt(portnumber);

      connection = new Socket(address, port); // server
      connection.setSoTimeout(soTimeout_);

      System.out.println("--* Connecting to " + connection.toString());
    }

    catch (UnknownHostException e) {
      System.out.println("UnknownHostException: " + e.getMessage());
    }

    catch (IOException e) {
      System.err.println("IO Exception: " + e.getMessage());
    }

    return connection;
  }

}
