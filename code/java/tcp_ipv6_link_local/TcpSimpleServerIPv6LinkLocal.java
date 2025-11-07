/**
 * Simple, non-blocking TCP server, IPv6 link-local, checks before reading.
 * The link-local address discovery is only for CS Linux lab machines.
 *
 * saleem, 11 Oct 2023
 * code check (sjm) - Oct 2023, Nov 2025
 */

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.net.InetAddress;
import java.net.NetworkInterface;
import java.net.ServerSocket;
import java.net.Socket;
import java.net.SocketTimeoutException;
import java.net.UnknownHostException;
import java.util.Enumeration;

public class TcpSimpleServerIPv6LinkLocal {

  static int           port_ = -1; // Might need to change this manually.
  static ServerSocket  server_;
  static int           sleepTime_ = 1000; // milliseconds
  static int           bufferSize_ = 80; // a line
  static int           soTimeout_ = 10; // 10 ms

  public static void main(String[] args) {

    com.sun.security.auth.module.UnixSystem unix = new com.sun.security.auth.module.UnixSystem();
    port_ = (int) unix.getUid(); // use unix uid as port number
    if (port_ < 1024) port_ += 1024; // adjust for reserved ports

    startServer();

    Socket       connection = null;
    InputStream  rx;
    OutputStream tx;

    while (connection == null) {
      try {
        connection = server_.accept(); // waits for connection
      }

      catch (SocketTimeoutException e) {
      // no incoming connection requests or data - just ignore
      }

      catch (IOException e) {
        System.err.println("IOException: " + e.getMessage());
        System.exit(0);
      }

    }

    try {
      rx = connection.getInputStream();
      tx = connection.getOutputStream();
      server_.close(); // no need to wait now

      System.out.println("New connection ... " + connection.toString());

      byte[] buffer = new byte[bufferSize_];
      int b = 0;
      while (b == 0) {

        buffer = new byte[bufferSize_];
        b = rx.read(buffer); // if connection closed by client, b == -1

        if (b > 0) {

          String s = new String(buffer);
          System.out.println("Received " + b + " bytes --> " + s);

          System.out.println("Sending back " + b + " bytes");
          tx.write(buffer, 0, b); // send data back to client
        }

        Thread.sleep(sleepTime_);

      } // while (b == 0)

      if (b == -1) System.out.println("<Connection closed by client>");
      else System.out.println("<Closing connection>");
      connection.close(); // finished

    } // try

    catch (SocketTimeoutException e) {
      // no incoming data - just ignore
    }
    catch (InterruptedException e) {
      System.err.println("Interrupted Exception: " + e.getMessage());
    }
    catch (IOException e) {
      System.err.println("IO Exception: " + e.getMessage());
    }
  }


  public static void startServer() {

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

      server_ = new ServerSocket(port_, 1, ip6); // make a socket
      server_.setSoTimeout(soTimeout_);
      System.out.println("--* Interface: " + nif.getName());
      System.out.println("--* Starting IPv6 server on link-local address: " + server_.toString());
    }

    catch (UnknownHostException e) {
      System.out.println("UnknownHostException: " + e.getMessage());
    }

    catch (IOException e) {
      System.err.println("IOException: " + e.getMessage());
    }
  } // startServer()

} // class
