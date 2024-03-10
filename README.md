# Trackermeta

This is a simple library crate that helps with parsing data from the website called [Mod Archive](https://modarchive.org), it recently had a big reborn update (v0.5.0).
It's now far more reliable, and parses the HTML very nicely, the code is readable and easier to use, the documentation is also superior, so if you've used this library in the past, **please upgrade!**

## Examples

Check out the [examples](examples) directory on the github repo for all examples using the library!

## Features

### Simplicity

One of the features is how simple using the library is, there is one struct,
`ModInfo`, and two functions on top of that, `get()` and `resolve_filename()`,
they get a module by their id or search for a string, respectively.

(please check documentation for more info)

### Infinity retry

This feature basically enables you to make the library retry infinitely
(http requests) regardless of errors until Modarchive gives in
