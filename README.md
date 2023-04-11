# Rustminder

Simple reminder written in Rust

# usage

To add an event to your remainder:
  1. create a file in "data/" with the rce extension
  2. add your entry following the format below
  3. ```cargo run```

# entry example

```
# this is a comment
# mind the separators = , ;
# spaces will be trimmed
# some slots are optional

# person = first_name, last_name, nickname ; birthday ; saint_day ; wedding_day
person = Santa, CLAUS, St Nicholas ; 25,12 ; 06,12 ;
# birthyear and wedding were omitted
# at least first_name or nickname must be provided, all other slots are optional
# birthday and wedding day take an optional year

# holiday = name ; begin ; end
holiday = Christmas ; 25,12
# recurring holiday, year and end are omitted
holiday = Easter ; 09,04,2023
# moving holiday, year must be set
holiday = Summer ; 01,07,2023 ; 31,08,2023
# spanning holiday, no slot is optional

# special = name ; date
special = IMPORTANT ; 04,07,2023
```

An entry can generate multiple events, for examples:
  1. a "person" entry with both birthday and wedding day will generate 2 events
  2. a "holiday" entry spanning 10 days will generate 10 events
  3. a "special" entry only generate 1 event

# future work

add thiserror crate to mix io::errors from reading files and <type>::Err from parsing
  1. [thiserror crate](https://docs.rs/thiserror/latest/thiserror/)
  2. [thiserror tutorial](https://youtu.be/g6WUHcyjsfc)
