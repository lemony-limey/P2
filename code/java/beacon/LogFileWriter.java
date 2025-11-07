/**
 * LogFileWriter - simple logging to local file.
 
 Saleem Bhatti <saleem@st-andrews.ac.uk>
 Nov 2025, code check (sjm55).
 Sep 2024, code check with java 21 on CS Linux Lab machines.
 Sep 2022, code check.
 Sep 2021, code check.
 Sep 2020, code check.
 Sep 2019, code check.
 Oct 2018, initial version.
 
 */

import java.io.File;
import java.io.FileWriter;
import java.text.SimpleDateFormat;
import java.util.Date;

public class LogFileWriter {

  public FileWriter fw;
  public SimpleDateFormat sdf;

  public LogFileWriter(String fileName) {
    File lf = new File(fileName);
    sdf = new SimpleDateFormat(new String("yyyyMMdd-HHmmss.SSS"));

    try {
      if (lf.exists()) fw = new FileWriter(fileName, true);
      else {
          lf.createNewFile();
          fw = new FileWriter(fileName, true);
      }
    }
    catch (Exception e) { System.out.println(e);}
  }

  public void writeLog(String logRequest) { writeLog(logRequest, false); }

  public void writeLog(String logRequest, Boolean stdout) {
    try {
      String now = sdf.format(new Date());
      String logEntry = new String(now.toString() + "| " + logRequest + "\n");
      fw.write(logEntry, 0, logEntry.length());
      fw.flush();
      if (stdout) { System.out.print(logEntry); }
    }
    catch (Exception e) { System.out.println(e); }
  }

}
