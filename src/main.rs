use uuid::Uuid;

const DEFAULT_SPARSE_INDEX_BLOCK_SIZE: i64 = 4096;

fn main() {
    println!("Hello, world!");
}

struct Project {
    id: Uuid,
    namespace: String,
}

struct EntityType {
    id: Uuid,
    project_id: Uuid,
    name: String,
}

struct Feature {
    id: Uuid,
    entity_type_id: Uuid,
    name: String,
    feature_timestamp_window: FeatureTimestampWindow,
    sparse_index_block_size: i64,
}

impl Feature {
    fn new(entity_type_id: Uuid, name: String, feature_timestamp_window: FeatureTimestampWindow, sparse_index_block_size: i64) -> Self {
        Self {
            id: Uuid::now_v7(),
            entity_type_id,
            name,
            feature_timestamp_window,
            sparse_index_block_size,
        }
    }

    fn with_defaults(entity_type_id: Uuid, name: String) -> Self {
        Self::new(
            entity_type_id,
            name,
            FeatureTimestampWindow::default(),
            DEFAULT_SPARSE_INDEX_BLOCK_SIZE,
        )
    }
}

#[derive(Default)]
enum FeatureTimestampWindow {
    Year,
    Month,
    #[default]
    Day,
    Hour,
}