# Using rust to parse jdeps output into recursive JSON

For work I need to pull out a feature into it's own repo so we can own the
lifecycle of this feature (e.g. deploying to lambda...). This implies that we'll
also need to identify all shared dependencies between our code path and the rest
of the original projet. Jdeps is a good tool for listing java dependencies
recursively, but I couldn't figure out how to organize the results into a
readable format: I just wanted to know what dependencies our particular code
path had.

jdeps output looks like so:

```text
software.amazon.awssdk.utils                       -> software.amazon.awssdk.utils                       software.amazon.awssdk.utils-2.17.209.jar
...
software.amazon.awssdk.utils.async                 -> java.lang                                          java.base
software.amazon.awssdk.utils.async                 -> java.lang.invoke                                   java.base
...
software.amazon.awssdk.utils.async                 -> software.amazon.awssdk.utils                       software.amazon.awssdk.utils-2.17.209.jar
software.amazon.awssdk.utils.async                 -> software.amazon.awssdk.utils.async                 software.amazon.awssdk.utils-2.17.209.jar
...
software.amazon.awssdk.utils.builder               -> software.amazon.awssdk.utils.builder               software.am
```

I wanted to be able to pass in my package name, and return a JSON object
representing the recurrsive decent into it's dependencies, like so:

``` json
{
  "name": "my.root.package",
  "internal": true,
  "deps": [
    {
      "name": "my.root.package.dep1",
      "internal": true,
      "deps": [...]
    },
    {
      "name": "org.external.dep1",
      "internal": false,
      "deps": [...]
    }
  ]
}
```

This helper bin takes in a file path to the jdeps output, and it takes in your
root package name, and it spits out json to stdout.
