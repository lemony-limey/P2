/**
 * Browses a file tree with a simple text based output and navigation.
 *
 * @author   <a href="https://saleem.host.cs.st-andrews.ac.uk/">Saleem Bhatti</a>
 * @version  1.5, 18 September 2024
 * Nov 2025, code check (sjm).
 * 18 Sep 2024, checked with java 21 on CS Lab Linux machines. 
 *
 */

import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.net.UnknownHostException; // used once network code is added
import java.text.SimpleDateFormat;
import java.util.Date;

public final class FileTreeBrowser {

  // user commands
  final static String quit     = new String(":quit");
  final static String help     = new String(":help");
  final static String services = new String(":services");
  final static String up       = new String("..");
  final static String list     = new String(".");
  final static String nodes    = new String(":nodes");
  final static String search   = new String(":search");
  final static String download = new String(":download");

  final static String propertiesFile = "filetreebrowser.properties";
  static Configuration configuration;
  static String rootPath = "";

  File   thisDir;  // this directory
  String thisDirName;  // name of this directory
  SimpleDateFormat sdf;

  public static String timestamp() {
    SimpleDateFormat sdf = new SimpleDateFormat(new String("yyyyMMdd-HHmmss.SSS"));
    return sdf.format(new Date());
  }

  /**
   * Create a path relative to the logical root directory.
   * 
   * @param pathName : a full/canonical path for a file/directory.
   */
  static String logicalPathName(String pathName)
  {
    String p = pathName.replace(rootPath, "");
    if (p == "") p = "/";
    return p;
  }

  /**
   * @param args : no args required
   */
  public static void main(String[] args) {

    configuration = new Configuration(propertiesFile);
    rootPath = getPathName(new File(configuration.rootDir));

    InputStream keyboard = System.in;
    String userCmd = new String(list);
    boolean quitBrowser = false;

    FileTreeBrowser ftb = new FileTreeBrowser(configuration.rootDir);
    ftb.printList();

    while(!quitBrowser) {

      System.out.print("\n[filename | '" + list + "' | '" + up + "' | '" + services + "' | '" + nodes + "' | '" + search + "' | '" + download + "' | '" + quit + "' | '" + help + "'] ");

      // what does the user want to do?
      while((userCmd = ByteReader.readLine(keyboard)) == null) {
        try { Thread.sleep(configuration.sleepTime); }
        catch (InterruptedException e) { } // Thread.sleep() - do nothing
      }

      // blank
      if (userCmd.isBlank()) { continue; }

      // quit
      if (userCmd.equalsIgnoreCase(quit)) { quitBrowser = true; }

      // help message
      else
      if (userCmd.equalsIgnoreCase(help)) { displayHelp(); }

      // service info
      else
      if (userCmd.equalsIgnoreCase(services)) { displayServices(); }

      // list files
      else
      if (userCmd.equalsIgnoreCase(list)) { ftb.printList(); }

      // move up directory tree
      else
      if (userCmd.equalsIgnoreCase(up)) {
        // move up to parent directory ...
        // but not above the point where we started!
        if (ftb.thisDirName.equals(rootPath)) {
          System.out.println("At root : cannot move up.\n");
        }
        else {
          String parent = ftb.thisDir.getParent();
          System.out.println("<<< " + logicalPathName(parent) + "\n");
          ftb = new FileTreeBrowser(parent);
        }
      }

      //  list discovered servers
      else
      if (userCmd.equalsIgnoreCase(nodes)) { nodes(); }

      else
      if (userCmd.equalsIgnoreCase(search)) { search(); }

      // download
      else
      if (userCmd.equalsIgnoreCase(download)) { download(); }

      else { // do something with pathname

        File f = ftb.searchList(userCmd);

        if (f == null) {
          System.out.println("Unknown command or filename: '" + userCmd + "'");
        }

        // act upon entered filename
        else {

          String pathName = getPathName(f);

          if (f.isFile()) { // print some file details
            System.out.println("file: " + logicalPathName(pathName));
            System.out.println("size: " + f.length());
          }

          else
          if (f.isDirectory()) { // move into to the directory
            System.out.println(">>> " + logicalPathName(pathName));
            ftb = new FileTreeBrowser(pathName);
          }

        } // (f == null)

      } // do something

    } // while(!quit)

  } // main()


  /**
   * Create a new FileTreeBrowser.
   *
   * @param pathName the pathname (directory) at which to start.
   */
  public FileTreeBrowser(String pathName) {
    if (pathName == "") { pathName = configuration.rootDir; }
    else // "." -- this directory, re-list only
    if (pathName.equals(list)) { pathName = thisDirName; }
    thisDir = new File(pathName);
    thisDirName = getPathName(thisDir);
  }


  /**
   * Print help message.
   */
  static void displayHelp() {

    String[] lines = {
      "--* Welcome to the simple FileTreeBrowser. *--",
      "* The display consists of:",
      "\t- The name of the current directory",
      "\t- The list of files (the numbers for the files are of no",
      "\t  significance, but may help you with debugging).",
      "* Files that are directories have trailing '" + File.separator + "'.",
      "* Use text entry to navigate the directory tree.",
      "\t.\t\tTo refresh the view of the current directory.",
      "\t..\t\tTo move up a directory level.",
      "\tfilename\tTo list file details (if it is a file) or to",
      "\t\t\tmove into that directory (if it is a directory name).",
      "\t:services\tTo list the services offered.",
      "\t:nodes\t\tTo list the other nodes discovered.",
      "\t:download\tTo download a file.",
      "\t:quit\t\tTo quit the program.",
      "\t:help\t\tTo print this message."
    };

    for(int i = 0; i < lines.length; ++i)
      System.out.println(lines[i]);

    return;
  }

  /**
   * Print config information.
   */
  static void displayServices() {

    String services = ":";
    services += "id=" + configuration.id + ":";
    services += "timestamp=" + timestamp() + ":";
    services += "search=" + configuration.search + ",";
    services += "download=" + configuration.download;
    services += ":";

    System.out.println(services);
  }

  static void nodes() { // TBC
    System.out.println("\n * nodes: TBC");
  }

  static void search() { // TBC
    System.out.println("\n * search: TBC");
  }

  static void download() { // TBC
    System.out.println("\n * download: TBC");
  }

  /**
   * List the names of all the files in this directory.
   */
  public void printList() {

    File[] fileList = thisDir.listFiles();

    System.out.println("\n+++  id: " + configuration.id);
    System.out.println("+++ dir: " + logicalPathName(getPathName(thisDir)));
    System.out.println("+++\tfilename:");
    for(int i = 0; i < fileList.length; ++i) {

      File f = fileList[i];
      String name = f.getName();
      if (f.isDirectory()) // add a trailing separator to dir names
          name = name + File.separator;
      System.out.println(i + "\t" + name);
    }
    System.out.println("+++");
  }

  String getParent() { return thisDir.getParent(); }

  /**
   * Search for a name in the list of files in this directory.
   *
   * @param name the name of the file to search for.
   */
  public File searchList(String name) {

    File found = null;

    File[] fileList = thisDir.listFiles();
    for(int i = 0; i < fileList.length; ++i) {

      if (name.equals(fileList[i].getName())) {
        found = fileList[i];
        break;
      }
    }

    return found;
  }


  /**
   * Get full pathname.
   *
   * @param f the File for which the pathname is required.
   */
  static public String getPathName(File f) {

    String pathName = null;

    try {
      pathName = f.getCanonicalPath();
    }
    catch (IOException e) {
      System.out.println("+++ FileTreeBrowser.pathname(): " + e.getMessage());
    }

    return pathName;
  }

}
