# DIRXPAND

A simple way to instantiate folder structures from easily written plain text. This allows you to create a folder structure once and reuse it easily.

The text used should be formatted like so:
```
file1.txt
file2.txt
extras/
    extra.txt
music/
    music.mp3
```
This will be expanded to become a folder with the above contents.

To make a folder, simply write the name followed by a slash. To make a file, simply write the name of the file. To put things in a folder, make sure they are indented past the folder (exact spacing irrelevant) and below it.

To use, simply open a plain text file with the executable, or (ideally) rename the file to have a `.dir` extension and always open with the executable, allowing double-clicking `.dir` files to quickly instantiate a folder.

If the target folder already exists, execution will fail and nothing will happen.