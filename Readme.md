# JSON Traverse

## What
This lib will parse the JSON string into composition of HashMaps and Vectors where JSON Object will be convert to HashMaps and JSON List will be the Vector. I decided to use Hashmap because it is easy to retrive the key and iterate through it without any fancy method.

## Why
I'm looking for a JSON tool in rust language that can able to easily retrive the key of the JSON object. I tried `serde`, may be I'm too dumb but I can't find the way to retrive the list or iterable of keys from JSON object, so I decided to implement JSON parser by myself.

## How
Since Rust is type strict, but still allow multiple types in enum, so I create an enum called JSON to support all of data types in JSON. The table below shows the relation between my enum and each of JSON data type.

| enum | JSON Data type|
|-------|---------|
| `JSON::Int`  | Number: `123` |
| `JSON::Flt*` | Number with floating points: `123.321` |
| `JSON::Str` | A JSON String datatype: `"Example String"` |
| `JSON::Lst` | A JSON List: `[ "Str1", 12, 13.1 ]` |
| `JSON::Obj` | An JSON Object: `{ "K1":12, "K2":false }` |
| `JSON::Bol` | A boolean data: `true, false` |
| `JSON::Nul` | A null data type: `null` |

## When
Currently, it is still in an alpha stage, I didn't finish the test yet, also the code so messy with a lot of duplication state ;-; (I use the concept of push down automata to implement this...) I promise I will do code cleanup in.... some day. Thus, I can't give you any clear date when this lib going to be on **crate.io**.

*PS: For the float data type, I still find a good way to merge with JSON::Int and renamed it to JSON::Num... but I still not sure which will be the best solution.