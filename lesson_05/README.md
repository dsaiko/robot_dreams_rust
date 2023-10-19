# Lesson 05
[back to main](../README.md)

https://robot-dreams-rust.mag.wiki/5-error-handling/index.html#homework

## Task

Implement Error handling and add CSV formatting

## Implementation

Main takes first argument as command and applies given transformation to the text input.

You can end the text input with Ctrl+D.

Works also with stdin:

    echo "text to transform" | ./lesson_5 uppercase
    cat file.txt | ./lesson_5 uppercase
    ./lesson_05 uppercase < file.txt
    ./lesson_05 csv < file.csv
    

## Sample transformation of CVS with multi line values

```text

┏━━━━━━━━━━━━━━━━━━━━━━━┯━━━━━━━━━━━┯━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┯━━━━━━━━━━━━━┯━━━━━━━━━┯━━━━━━━┓
┃ First Name            │ Last Name │ Address                          │ City        │ Country │ ZIP   ┃
┃                       │           │                                  │             │ and     │       ┃
┃                       │           │                                  │             │ State   │       ┃
┡━━━━━━━━━━━━━━━━━━━━━━━┿━━━━━━━━━━━┿━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┿━━━━━━━━━━━━━┿━━━━━━━━━┿━━━━━━━┩
│ John2                 │ Doe       │ 120 jefferson st.                │ Riverside   │ NJ      │ 08075 │
├───────────────────────┼───────────┼──────────────────────────────────┼─────────────┼─────────┼───────┤
│ Jack                  │ McGinnis  │ 220 hobo Av.                     │ Phila       │ PA      │ 09119 │
├───────────────────────┼───────────┼──────────────────────────────────┼─────────────┼─────────┼───────┤
│ John "Da Man"         │ Repici    │ 120 Jefferson St.                │ Riverside   │ NJ      │ 08075 │
├───────────────────────┼───────────┼──────────────────────────────────┼─────────────┼─────────┼───────┤
│ Stephen               │ Tyler     │ 7452 Terrace "At the Plaza" road │ SomeTown    │ SD      │ 91234 │
├───────────────────────┼───────────┼──────────────────────────────────┼─────────────┼─────────┼───────┤
│                       │ Blankman  │                                  │ SomeTown    │ SD      │       │
├───────────────────────┼───────────┼──────────────────────────────────┼─────────────┼─────────┼───────┤
│ Joan "the bone", Anne │ Jet       │ 9th,                             │ Desert City │ CO      │ 00123 │
│                       │           │ at Terrace plc with long         │             │         │       │
│                       │           │ description                      │             │         │       │
└───────────────────────┴───────────┴──────────────────────────────────┴─────────────┴─────────┴───────┘
```