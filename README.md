# what_stream

simply provide this a query to search for and it'll list programming streams that match

a query is a space separated list of words.

e.g. `"c++ rust gamedev"`

the `query` will match words in the stream title. the query is case-insensitive. so `opengl` will match `OpenGL`

## requirements

you must set `WHAT_STREAM_CLIENT_ID` and `WHAT_STREAM_BEARER_OAUTH` to their appropriate values.

you can get these from the Twitch developer console (_hint_ or your browser if you know where to look)

## example

this'll look for streams with 'c++' or 'python' or 'rust' in their title.

`> what_stream c++ python rust`

```
┌── python
├ https://twitch.tv/Pydathon
├ Python, scraping et data analyse :) !discord
├ started 1 hours 21 minutes ago, 3 watching
│
├ https://twitch.tv/beginbot
├ Vim Day! + [Go, Python, Linux] !me AMA
└ started 1 hours 0 minutes ago, 53 watching

┌── rust
├ https://twitch.tv/museun
├ random projects in Rust
├ started 2 hours 1 minutes ago, 2 watching
│
├ https://twitch.tv/Brookzerker
├ Building a Pitfall clone in Rust + GGEZ
├ started 1 hours 22 minutes ago, 4 watching
│
├ https://twitch.tv/togglebit
├ Rust, Rantings and Ramblings: Friday fun! (whatever that means)
└ started 8 hours 38 minutes ago, 53 watching
```

**note** if a query does not match, it will not be displayed.

also, try using one of the `--style` options

## usage

```
what_stream 0.2.0

USAGE:
    what_stream [flags] [query ..]

FLAGS:
    -h, --help              show the help message
    -v, --version           show the current version
    -s, --sort <col, dir?>  sort by <col> in the optional <dir>
    -t, --style <style>     what type of rendering style to use
    -j, --json              dumps the results as json

SORTING:
    available columns:
     - name (the default)
     - viewers
     - uptime

    available directions:
     - descending (the default)
     - desc (shorthand)

     - ascending
     - asc (shorthand)

QUERY:
    query is a space separated list of 'tags' to filter by.
    the tags are case-insensitive and will match 'words' in the stream title
    e.g. 'opengl' will match 'OpenGL' in a title: "making a game with OpenGL"

STYLES:
    - box (the default)
    - fancy
    - none

NOTES:
    if NO_COLORS is set, the colors are disabled

    the following environment variables must be set:
    WHAT_STREAM_CLIENT_ID
    - this your Twitch api Client-ID

    WHAT_STREAM_BEARER_OAUTH
    - this is an associated bearer OAuth from the Client-ID
```

## license

[0BSD](./LICENSE.txt)
