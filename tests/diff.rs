use serde_json::json;
use json_diff_rs::{ArrayIndex, ElementPath, Filter};

#[test]
fn ignore_field() {
    let user_1 = json!({
        "user": "John",
        "address": {
            "city": "Astana",
            "zip": 123,
        },
        "age": 33
    });

    let user_2 = json!({
        "user": "Joe",
        "address": {
            "city": "Boston",
            "zip": 312,
        }
    });

    let diff = json_diff_rs::objects(
        serde_json::from_value(user_1).unwrap(),
        serde_json::from_value(user_2).unwrap(),
        vec![
            Filter::Ignore(vec![ElementPath::Key("user".to_string())]),
            Filter::Ignore(vec![ElementPath::Key("age".to_string())]),
            Filter::Ignore(vec![
                ElementPath::Key("address".to_string()),
                ElementPath::Key("city".to_string())
            ]),
            Filter::Ignore(vec![
                ElementPath::Key("address".to_string()),
                ElementPath::Key("zip".to_string())
            ]),
        ],
        vec![]
    );

    assert_eq!(true, diff.is_none(), "diff should be None, but got: {:?}", diff);
}

#[test]
fn ignore_object_in_array() {
    let data_1 = json!({
        "data": [
            {"id": 1, "created_at": "2024"}
        ]
    });

    let data_2= json!({
        "data": [
            {"id": 1, "created_at": "2023"},
            {"id": 2, "created_at": "2025"}
        ]
    });

    let diff = json_diff_rs::objects(
        serde_json::from_value(data_1).unwrap(),
        serde_json::from_value(data_2).unwrap(),
        vec![
            // data._.created_at
            Filter::Ignore(vec![
                ElementPath::Key("data".to_string()),
                ElementPath::ArrayIndex(ArrayIndex::All),
                ElementPath::Key("created_at".to_string()),
            ])
        ],
        vec![]
    );

    assert_eq!(true, diff.is_none(), "diff should be None, but got {:?}", diff)
}

// #[test]
// fn kitchen_sink() {
//     let a = json!({
//       "A": "a",
//       "B": "a",
//       "D": 1,
//       "E": 1,
//       "F": [],
//       "G": ["a", "a"],
//     });
//     let b = json!({
//       "A": "a",
//       "C": "b",
//       "D": 2,
//       "E": "1",
//       "F": [true],
//       "G": ["a", "ab"],
//     });
//
//     let diff = serde_json_diff::objects(
//         serde_json::from_value(a).unwrap(),
//         serde_json::from_value(b).unwrap(),
//     );
//
//     insta::assert_snapshot!(serde_json::to_string_pretty(&diff).expect("couldn't pretty"));
// }

// #[test]
// fn types() {
//     let left = json!("a");
//     let right = json!(true);
//
//     let diff = serde_json_diff::values(left, right);
//
//     insta::assert_snapshot!(serde_json::to_string_pretty(&diff).unwrap());
// }
//
// #[test]
// fn entries() {
//     let left = json!({
//         "a": false,
//         "c": 1,
//     });
//     let right = json!({
//         "b": false,
//         "c": 2,
//     });
//
//     let diff = serde_json_diff::objects(
//         serde_json::from_value(left).unwrap(),
//         serde_json::from_value(right).unwrap(),
//     );
//
//     insta::assert_snapshot!(serde_json::to_string_pretty(&diff).unwrap());
// }
//
// #[test]
// fn arrays() {
//     let source = json!([]);
//     let target = json!([true]);
//
//     let diff = serde_json_diff::arrays(
//         serde_json::from_value(source).unwrap(),
//         serde_json::from_value(target).unwrap(),
//     );
//
//     assert!(matches!(diff, Some(ArrayDifference::Shorter { .. })));
//
//     let source = json!([true]);
//     let target = json!([]);
//
//     let diff = serde_json_diff::arrays(
//         serde_json::from_value(source).unwrap(),
//         serde_json::from_value(target).unwrap(),
//     );
//
//     assert!(matches!(diff, Some(ArrayDifference::Longer { .. })));
//
//     let source = json!([true]);
//     let target = json!([false]);
//
//     let diff = serde_json_diff::arrays(
//         serde_json::from_value(source).unwrap(),
//         serde_json::from_value(target).unwrap(),
//     );
//
//     assert!(matches!(diff, Some(ArrayDifference::PairsOnly { .. })));
//
//     let source = json!([true]);
//     let target = json!([true]);
//
//     let diff = serde_json_diff::arrays(
//         serde_json::from_value(source).unwrap(),
//         serde_json::from_value(target).unwrap(),
//     );
//
//     assert!(diff.is_none());
// }
