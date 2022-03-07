# networkplan-renderer

Uses the xml-output of `nmap` to generate a (drawio)[https://app.diagrams.net/] diagram.

## Usage

Run `nmap` with the `-oX` parameter to generate a xml-file


Example Commandline:

```sh
sudo nmap -oX output.xml -A -T4 -p- 10.129.0.1/16
```

Point the program to the xml file.

Just open the `export.drawio` using `drawio`.
