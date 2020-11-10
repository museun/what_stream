# what_stream

simply provide this a query to search for and it'll list programming streams that match

a query is a comma separated list of words.

e.g. `"c++,rust,gamedev"`

the `query` will match words in the stream title. the query is case-insensitive. so `opengl` will match `OpenGL`

## requirements

you must set `WHAT_STREAM_CLIENT_ID` and `WHAT_STREAM_BEARER_OAUTH` to their appropriate values.

you can get these from the Twitch developer console (_hint_ or your browser if you know where to look)

## usage


this'll look for streams with 'c++' or 'python' or 'rust' in their title.

`> what_stream "c++,python,rust"`

```
streams for 'c++'
   | uptime |                link                | title
--------------------------------------------------------
51 |    26m | https://twitch.tv/DaveChurchill    | Computer Science 4300 - C++ Game Programming - type !course or !website for more info
 9 |  4h 1m | https://twitch.tv/Moscowwbish      | Tile Map Editor | C++ | OpenGL | ImGUI
 3 | 2h 35m | https://twitch.tv/MariusUrbelis    | C++ OpenGL, New emote! ❤️
 1 |  5h 8m | https://twitch.tv/SomewhatAccurate | Refactoring and cleaning. They call me the janitor. C++ / XML / Lua / OpenGL / GLFW / Glad / CMake / Github

streams for 'python'
   | uptime |                link                | title
--------------------------------------------------------
 4 | 2h 48m | https://twitch.tv/PatricPuola      | Man vs Python
 2 |  4h 7m | https://twitch.tv/kimtekk          | Payroll System [ Day 2 ] [ Python ] 1st Project - Creating Payroll System with Reasonable GUI
```

## license

[0BSD](./LICENSE.txt)
