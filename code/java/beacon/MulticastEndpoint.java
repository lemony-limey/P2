/**
 * IPv4/IPv6 multicast socket goop wrapper.

  Saleem Bhatti <saleem@st-andrews.ac.uk>
  Nov 2025, code check (sjm55).
  Sep 2024, code check with java 21 on CS Linux Lab machines.
  Sep 2022, updated code for dual-stack IPv4/IPv6.
  Sep 2021, updated code to remove deprecated API usage in Java 17 LTS.
  Sep 2020, code check.
  Sep 2019, code check.
  Oct 2018, initial version.

  This encapsulates the process of setting up an IPv4/IPv6 multicast
  communication endpoint, as well as sending and receiving from that
  endpoint.
*/

// Java 21 LTS from Sep 2024
// https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/net/MulticastSocket.html

import java.io.IOException;
import java.net.DatagramPacket;
import java.net.InetAddress;
import java.net.InetSocketAddress;
import java.net.MulticastSocket;
import java.net.SocketException;
import java.net.SocketTimeoutException;
import java.net.StandardSocketOptions;

public class MulticastEndpoint
{
  MulticastSocket mSocket;
  InetAddress mInetAddr4;
  InetAddress mInetAddr6;
  InetSocketAddress mGroup4;
  InetSocketAddress mGroup6;
  Configuration c;

  public enum PktType { none, ip4, ip6 } // for dual stack flexibility

  /**
   * A multicast communication end-point, dual stack (IPv4 and IPv6).
   * It is possible to have sockets that are only either IPv4 or IPv6,
   * but this is an example of how to have both.
   * @param configuration : Configuration object, config info.
   */
  MulticastEndpoint(Configuration configuration)
  {
    c = configuration;

    try {

      mSocket = new MulticastSocket(c.mPort);
      mSocket.setOption(StandardSocketOptions.IP_MULTICAST_LOOP, c.loopback);
      mSocket.setReuseAddress(c.reuseAddr); // re-use of addr on this host
      mSocket.setTimeToLive(c.mTTL); // maximum number of hops to send
      mSocket.setSoTimeout(c.soTimeout); // non-blocking socket

      if (c.mAddr4 != null && !c.mAddr4.equalsIgnoreCase(c.zeroAddr)) {
        mInetAddr4 = InetAddress.getByName(c.mAddr4);
        mGroup4 = new InetSocketAddress(mInetAddr4, c.mPort);
      }

      if (c.mAddr6 != null && !c.mAddr6.equalsIgnoreCase(c.zeroAddr)) {
        mInetAddr6 = InetAddress.getByName(c.mAddr6);
        mGroup6 = new InetSocketAddress(mInetAddr6, c.mPort);
      }

      c.log.writeLog("using interface " + c.nif.getName());

      if (mGroup4 == null && mGroup6 == null) {
        c.log.writeLog("No multicast group defined -- exiting");
        System.err.println("No multicast group defined -- exiting");
        System.exit(0);
      }

    } // try
  
    catch (SocketException e) {
      System.out.println("MulticastEndpoint(): " + e.getMessage());
    }

    catch (IOException e) {
      System.out.println("MulticastEndpoint(): " + e.getMessage());
    }
  }

  /**
   * Join multicast group(s).
   */
  void join()
  {
    try {
      if (mGroup4 != null) {
        mSocket.joinGroup(mGroup4, c.nif);
        c.log.writeLog("joined IPv4 multicast group " + mGroup4.toString(), true);
      }
      if (mGroup6 != null) {
        mSocket.joinGroup(mGroup6, c.nif);
        c.log.writeLog("joined IPv6 multicast group " + mGroup6.toString(), true);
      }
    }
    catch (IOException e) {
      System.out.println("MulticastEndpoint.join(): " + e.getMessage());
    }
  }

  
  /**
   * Leave multicast group(s).
   */
  void leave()
  {
    if (mSocket == null) return;

    try {
      if (mGroup4 != null) {
        mSocket.leaveGroup(mGroup4, c.nif);
        c.log.writeLog("left IPv4 multicast group", true);
      }
      if (mGroup6 != null) {
        mSocket.leaveGroup(mGroup6, c.nif);
        c.log.writeLog("left IPv6 multicast group", true);
      }

      mSocket.close();

    }
    catch (IOException e) {
      System.out.println("MulticastEndpoint.leave(): " + e.getMessage());
    }
  }

  /** 
   * Receive a multicast packet from a dual-stack socket.
   * @param b : byte array for received data
   * @return : type of packet received, IPv4 or IPv6
   */
  PktType rx(byte[] b)
  {
    if (b == null || mSocket == null) return PktType.none;

    PktType p = PktType.none;
  
    try {
      DatagramPacket d = new DatagramPacket(b, b.length);

      mSocket.receive(d);
      final int l = d.getLength();

      if (l > 0) {
        int addrLen = d.getAddress().getAddress().length;
        if (addrLen ==  4) p = PktType.ip4; //  4 bytes, IPv4
        if (addrLen == 16) p = PktType.ip6; // 16 bytes, IPv6
      }
    }
    catch (SocketTimeoutException e) {
      // do nothing
    }
    catch (IOException e) {
      System.out.println("MulticastEndpoint.rx(): IOException - " + e.getMessage());
    }

    return p;
  }

  /**
   * @param p : select IPv4 or IPv6
   * @param b : bytes to send
   * @return true / false for successful / non-successful transmission
   */
  boolean tx(PktType p, byte b[])
  {
    if (p == PktType.none || b == null || mSocket == null) return false;

    InetSocketAddress mGroup = null;
  
    if (p == PktType.ip4) {
      if (mGroup4 == null) return false;
      mGroup = mGroup4;
    }

    else if (p == PktType.ip6) {
      if (mGroup6 == null) return false;
      mGroup = mGroup6;
    }
  
    boolean done;
    DatagramPacket d;

    done = false;
    try {
      d = new DatagramPacket(b, b.length, mGroup);
      mSocket.send(d);
      done = true;
    }

    catch (SocketTimeoutException e) {
      System.out.println("MulticastEndpoint.tx(): timeout on send - " + e.getMessage());
    }
    catch (SocketException e) {
      System.out.println("MulticastEndpoint.tx(): SocketException - " + e.getMessage());
    }
    catch (IOException e) {
      System.out.println("MulticastEndpoint.tx(): IOException - " + e.getMessage());
    }

    return done;
  }

} // class MulticastEndpoint

