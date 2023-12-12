# JSON Traverse

## What
This lib will parse the JSON string into a composition of HashMaps and Vectors where JSON Object will be convert to HashMaps and JSON List will be the Vector. I decided to use Hashmap because it is easy to retrive the key and iterate through it without any fancy method.

## Changelog
- 12.12.2023: Adding new function `pretty_print()`. Used for print the beautiful JSON string.

## Why
I'm looking for a JSON parsing lib in Rust language that easily to retreive list JSON Object keys. I tried `serde`, maybe I'm too dumb but I can't find any simple way to retreive it, so I decided to implement a JSON parser.

## How
Since Rust is type strict but still allows multiple types in enum, I create an enum called `JSON` to support all of data types in JSON. The table below shows the relation between my enum and each of JSON data type.

| enum | JSON Data type|
|-------|---------|
| `JSON::Int`  | Number: `123` |
| `JSON::Flt*` | Number with floating points: `123.321` |
| `JSON::Str` | A JSON String datatype: `"Example String"` |
| `JSON::Lst` | A JSON List: `[ "Str1", 12, 13.1 ]` |
| `JSON::Obj` | An JSON Object: `{ "K1":12, "K2":false }` |
| `JSON::Bol` | A boolean data: `true, false` |
| `JSON::Nul` | A null data type: `null` |

For the algorithm of this lib, I got some inspiration from PDA (Push Down Automata) and I tried to code this by myself, so please do not expect any excellent coding style or idiomatic way of rust or 100% correctness of PDA.

## When
Currently, it is still in the alpha stage. I didn't finish the test yet, also the code so messy with a lot of duplication states ;-; I promise I will do code cleanup in.... some day, but I can't give you any precise date when this lib going to be on **crate.io**.

### *PS: 
- For the float data type, I still find a good way to merge with `JSON::Int` and renamed it to `JSON::Num`... but I still not sure which will be the best solution.
- all of `panics` is going to be remove soon.

