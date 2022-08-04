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
    "circular_with": [],
  "deps": [
    {
      "name": "my.root.package.dep1",
      "circular_with": ["depa", "depb"],
      "deps": [...]
    },
    {
      "name": "org.external.dep1",
      "circular_with": [],
      "deps": [...]
    }
  ]
}
```

This helper bin takes in a file path to the jdeps output, and it takes in your
root package name, and it spits out json to stdout.

NOTE: Right now I use recursion, so the stack overflows for large deps :(. I
would like to figure out how to get around that, but the recursive function is
so logical. I added an `-includes` arg which allows you only to include the
packages you want in the json. Passing this flag allows me to avoid the stack
overflow for my project, since I only really need to know about shared
_internal_ deps.

UPDATE: Tried a non-recursive version to get around the stack frame problem. But
now with a long/complex dep file, the main loop slows to a crawl, pretty sure
it's the copying of the recursive data structure. My next attempt I'll try
reference counting rather than cloning the whole recursive data structure. But
this works I think? Hard to tell exactly. To me the non-recursive is so much
harder to follow. Maybe it's wrong right now.
