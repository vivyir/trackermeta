# Trackermeta
This is a simple library and a small utility crate that helps with parsing
data from the website called [Modarchive](https://modarchive.org), the
utility searches modarchive for the filename provided, gets the most likely
result, extracts module id and then gets the full details for it as a single
csv record which the structure of it can be seen in the lib.rs file in the
first function of the trackermeta module.

The src/main.rs includes a full example on how to use the library which is
only 17 lines of code!

## Features

### Infinity retry
This feature basically enables you to make the library retry infinitely
regardless of errors until Modarchive gives in

### Overriding the default values
This library functions by using stable "anchor" points to start from and
extract meta-data which are hardcoded in the source but since it's better
to future proof in case of an event like a small design change in Modarchive
there is a way to override the main anchor points without needing to update
the whole program, and that is an extremely simple config file which is
enabled by the "**overridable**" feature and is  located depending on the 
platform, using the crate platform-dirs to determine the config file folder
which your program should modify in order to change anchor values, on linux
for example its located at:

`/home/user/.config/trackermeta/line-overrides`

in header-less csv and is read as "module\_filename\_line, module\_info\_line, 
module\_download\_line (the download count)" if you're still unsure of what
they are view the source page on an unnominated module of modarchive and
check out the lines which are hardcoded in the source-code, for the nominated
modules these are all raised by 6 since the nomination badge adds 6 lines
to the source page, which also has an anchor but since there haven't been
any problems even after the small shift which added the scenesat mirror banner
i haven't included them in the overrides but its very easy to do so if need be
