// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0



use super::*;
use proptest::prelude::*;
use schemadb::schema::assert_encode_decode;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10,
        .. ProptestConfig::default()
    })]
    #[test]
    fn test_encode_decode(
        block_id in any::<HashValue>(),
        pos_state in any::<PosState>(),
    ) {
        assert_encode_decode::<PosStateSchema>(&block_id, &pos_state);
    }
}
