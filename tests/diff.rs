use serde_json::json;
use json_diff_rs::{ArrayIndex, DiffBuilder, PathElement, IgnorePath};


#[test]
fn equal_objects() {
    let obj1 = json!({
        "string": "b",
        "int": 1,
        "float": 1.0,
        "bool": true,
        "int_array": [1, 2, 3],
        "float_array": [1.0, 2.0, 3.0],
        "bool_array": [true, false, false],
        "string_array": ["foo", "bar"],
        "empty_array": [],
        "null": null,
        "object": {
            "string": "c",
            "int": 1,
            "float": 1.0,
            "bool": true,
            "array": [1, 2, 3],
            "null": null,
            "object": {
                "string": "d",
            }
        },
    });

    let obj2 = json!({
        "string": "b",
        "int": 1,
        "float": 1.0,
        "bool": true,
        "int_array": [1, 2, 3],
        "float_array": [1.0, 2.0, 3.0],
        "bool_array": [true, false, false],
        "string_array": ["foo", "bar"],
        "empty_array": [],
        "null": null,
        "object": {
            "string": "c",
            "int": 1,
            "float": 1.0,
            "bool": true,
            "array": [1, 2, 3],
            "null": null,
            "object": {
                "string": "d",
            }
        },
    });

    let diff = DiffBuilder::default().source(obj1).target(obj2).build().unwrap();
    let diff = diff.compare();
    assert_eq!(true, diff.is_none(), "diff should be None, but got: {:?}", diff);
}


#[test]
fn ignore_fields() {
    let user_1 = json!({
        "user": "John",
        "address": {
            "city": "Astana",
            "zip": 123,
        },
        "animals": ["dog", "cat"],
        "object_array": [{"a": "b", "c": "d"}],
        "optional_array": [],
        "target_missing_value": 1,
    });

    let user_2 = json!({
        "user": "Joe",
        "address": {
            "city": "Boston",
            "zip": 312,
        },
        "animals": ["dog", "cat"],
        "object_array": [{"a": "3", "c": "d"}],
        "optional_array": null,
    });

    let diff = DiffBuilder::default()
        .ignore_path("user")
        .ignore_path("address.city")
        .ignore_path("address.zip")
        .ignore_path("object_array._.a")
        .ignore_path_with_missing("target_missing_value", true)
        .equate_empty_arrays(true)
        .source(user_1)
        .target(user_2)
        .build()
        .unwrap();

    let diff = diff.compare();

    assert_eq!(true, diff.is_none(), "diff should be None, but got: {:?}", diff);
}

#[test]
fn approx_float_eq() {
    let obj1 = json!({
        "float": 1.34
    });

    let obj2 = json!({
        "float": 1.341
    });

    let diff = json_diff_rs::DiffBuilder::default()
        .approx_float_eq_epsilon(0.001)
        .source(obj1).target(obj2).build().unwrap();

    let diff = diff.compare();

    assert_eq!(true, diff.is_none(), "diff should be None, but got: {:?}", diff);
}
