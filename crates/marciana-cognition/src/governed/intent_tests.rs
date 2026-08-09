use querygraph_memory::cognition::CognitionFieldMapping;
use typesec_memory::MemoryId;

use super::super::CognitionBindingError;
use super::{cognition_field_mapping_digest, cognition_source_selection_digest};

#[test]
fn source_selection_digest_is_order_independent_but_duplicate_safe() {
    let first = MemoryId::from_string("memory-a");
    let second = MemoryId::from_string("memory-b");

    let first_digest =
        cognition_source_selection_digest(&[first.clone(), second.clone()]).expect("first digest");
    assert_eq!(
        first_digest,
        "sha256:c36189131bd3ba1eda7cad373075380ef079aa9b37db1a479bf7e6a8be1b5f13"
    );
    assert_eq!(
        first_digest,
        cognition_source_selection_digest(&[second, first]).expect("second digest"),
    );
    assert_eq!(
        cognition_source_selection_digest(&[
            MemoryId::from_string("memory-a"),
            MemoryId::from_string("memory-a")
        ]),
        Err(CognitionBindingError::InvalidSourceSelection),
    );
}

#[test]
fn field_mapping_digest_rejects_ambiguous_projection() {
    let canonical = CognitionFieldMapping {
        id: "memory_id".into(),
        text: "memory_text".into(),
        valid_from: "valid_from".into(),
    };
    assert_eq!(
        cognition_field_mapping_digest(&canonical).expect("canonical mapping digest"),
        "sha256:9a3e565c0a78ac242de36b110f98191518931118390c4a6dec08956082461b1c"
    );

    let mapping = CognitionFieldMapping {
        id: "memory_id".into(),
        text: "memory_id".into(),
        valid_from: "valid_from".into(),
    };

    assert_eq!(
        cognition_field_mapping_digest(&mapping),
        Err(CognitionBindingError::InvalidProjection),
    );
}
