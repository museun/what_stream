# what_stream

simply provide this a query to search for and it'll list programming streams that match

a query is a space separated list of words.

e.g. `"c++ rust gamedev"`

the `query` will match words in the stream title. the query is case-insensitive. so `opengl` will match `OpenGL`

## requirements

you must set `WHAT_STREAM_CLIENT_ID` and `WHAT_STREAM_BEARER_OAUTH` to their appropriate values.

you can get these from the Twitch developer console (_hint_ or your browser if you know where to look)

## usage

this'll look for streams with 'c++' or 'python' or 'rust' in their title.

`> what_stream c++ python rust"`

```
streams for 'c++'
   | uptime |                link                | title
--------------------------------------------------------
 3 | 2h 14m | https://twitch.tv/qm0auber         | C++: Dalga Boyu Temelli Renklendirme ve Isin Izleyici(Ray Tracer) 3
 1 | 8h 34m | https://twitch.tv/SomewhatAccurate | Refactoring and cleaning. They call me the janitor. C++ / XML / Lua / OpenGL / GLFW / Glad / CMake / Github

streams for 'python'
   | uptime |                link                | title
--------------------------------------------------------
 1 |    11m | https://twitch.tv/mrdonbrown       | [Python] Django custom auth

streams for 'rust'
   | uptime |                link                | title
--------------------------------------------------------
65 | 2h 11m | https://twitch.tv/rhymu8354        | 0423 -- Rust: WebSockets Autobahn Testsuite
 2 |    30m | https://twitch.tv/museun           | random projects in Rust

```

### help

```
what_stream 0.1.0

USAGE:
    what_stream [flags] [query ..]

FLAGS:
    -h, --help              show the help message
    -v, --version           show the current version
    -s, --sort <col, dir?>  sort by <col> in the optional <dir>
    -c, --column <name>     enable this column
    -j, --json              dumps the results as json

SORTING:
    available columns: viewers, uptime, name
    available directions: ascending, descending (the default)

QUERY:
    query is a space separated list of 'tags' to filter by.
    the tags are case-insensitive and will match 'words' in the stream title
    e.g. 'opengl' will match 'OpenGL' in a title: "making a game with OpenGL"

COLUMNS:
    available visible columns: viewers, uptime, name, title
    if -c,--columns is not used then all columns are visible
    otherwise, only the specified columns are visible
    e.g. `-c name -c link` will only show the `name` and `link` columns

NOTES:
    the following environment variables must be set:

    WHAT_STREAM_CLIENT_ID
    - your Twitch api Client-ID

    WHAT_STREAM_BEARER_OAUTH
    - an associated bearer OAuth from the Client-ID
```

## license

[0BSD](./LICENSE.txt)
